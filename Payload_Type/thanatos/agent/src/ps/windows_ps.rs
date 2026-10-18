//! This module is only imported when targeting Windows hosts
use crate::ps::ProcessListingEntry;
use crate::utils::windows::Handle;
use chrono::DateTime;
use serde::Deserialize;
use std::error::Error;
use std::result::Result;
use winapi::{
    ctypes::c_void,
    shared::{
        minwindef::{FALSE, FILETIME, TRUE},
        winerror::ERROR_NO_MORE_FILES,
    },
    um::{
        errhandlingapi::GetLastError,
        handleapi::INVALID_HANDLE_VALUE,
        processthreadsapi::{GetProcessTimes, OpenProcess, OpenProcessToken},
        securitybaseapi::{GetSidSubAuthority, GetSidSubAuthorityCount, GetTokenInformation},
        tlhelp32::{
            CreateToolhelp32Snapshot, Module32First, Process32First, Process32Next, MODULEENTRY32,
            PROCESSENTRY32, TH32CS_SNAPMODULE, TH32CS_SNAPPROCESS,
        },
        winbase::LookupAccountSidA,
        winnt::{
            TokenIntegrityLevel, TokenUser, PROCESS_QUERY_INFORMATION, PSID, SID_NAME_USE,
            TOKEN_MANDATORY_LABEL, TOKEN_QUERY, TOKEN_USER,
        },
        wow64apiset::IsWow64Process,
    },
};
use wmi::{COMLibrary, WMIConnection};

/// Get the list of process information
pub fn process_info() -> Result<Vec<ProcessListingEntry>, Box<dyn Error>> {
    // Vec to hold the process infomation
    let mut listing: Vec<ProcessListingEntry> = Vec::new();

    // Grab a snapshot of all the running processes
    let h_snapshot_process =
        Handle::new(unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) }).ok_or_else(
            || {
                std::io::Error::other(format!("Failed to open process snapshot {:?}", unsafe {
                    GetLastError()
                }))
            },
        )?;

    // Create the object to store the process PE information
    let mut pe_entry = PROCESSENTRY32 {
        dwSize: std::mem::size_of::<PROCESSENTRY32>() as u32,
        ..Default::default()
    };

    // Create the object to store the modules entry information
    let mut mod_entry = MODULEENTRY32 {
        dwSize: std::mem::size_of::<MODULEENTRY32>() as u32,
        ..Default::default()
    };

    // Create the struct for the first process entry
    let mut first_entry = ProcessListingEntry {
        process_id: pe_entry.th32ProcessID,
        architecture: "".to_string(),
        parent_process_id: Some(pe_entry.th32ParentProcessID),
        ..Default::default()
    };

    // Grab the information for the first process
    if unsafe { Process32First(*h_snapshot_process, &mut pe_entry) } == TRUE {
        // Query the process token
        if let Some(handle) = Handle::new(unsafe {
            OpenProcess(PROCESS_QUERY_INFORMATION, FALSE, pe_entry.th32ProcessID)
        }) {
            // Populate the `ProcessListingEntry` with the process information
            first_entry.architecture = get_architecture(*handle).unwrap_or_default();
            first_entry.name = get_proc_name(&pe_entry);
            first_entry.start_time = get_start_time(*handle);

            let mut token_handle = INVALID_HANDLE_VALUE;
            if unsafe { OpenProcessToken(*handle, TOKEN_QUERY, &mut token_handle) } != FALSE {
                let token_handle = Handle::from(token_handle);
                first_entry.user = get_proc_user(*token_handle);
                first_entry.integrity_level = get_integrity_level(*token_handle);
            }
        }

        // Grab a snapshot of the process' modules
        if let Some(h_snapshot_module) = Handle::new(unsafe {
            CreateToolhelp32Snapshot(TH32CS_SNAPMODULE, pe_entry.th32ProcessID)
        }) {
            if unsafe { Module32First(*h_snapshot_module, &mut mod_entry) } == TRUE {
                // Add the bin path to the process listing information
                first_entry.bin_path = unsafe {
                    Some(
                        std::ffi::CStr::from_ptr(mod_entry.szExePath.as_ptr())
                            .to_string_lossy()
                            .to_string(),
                    )
                };
            }
        }

        // Append the process information to the listing
        listing.push(first_entry);
    }

    // Loop over all running processes
    loop {
        // Grab the next process in the snapshot
        if unsafe { Process32Next(*h_snapshot_process, &mut pe_entry) } == FALSE {
            // Check if there are no more processes to enumerate
            if unsafe { GetLastError() } == ERROR_NO_MORE_FILES {
                break;
            }
            continue;
        }

        // Create the listing entry struct
        let mut entry = ProcessListingEntry {
            process_id: pe_entry.th32ProcessID,
            architecture: "".to_string(),
            parent_process_id: Some(pe_entry.th32ParentProcessID),
            ..Default::default()
        };

        // Open a handle to the process token
        if let Some(handle) = Handle::new(unsafe {
            OpenProcess(PROCESS_QUERY_INFORMATION, TRUE, pe_entry.th32ProcessID)
        }) {
            entry.architecture = get_architecture(*handle).unwrap_or_default();
            entry.name = get_proc_name(&pe_entry);
            entry.start_time = get_start_time(*handle);

            let mut token_handle = INVALID_HANDLE_VALUE;
            if unsafe { OpenProcessToken(*handle, TOKEN_QUERY, &mut token_handle) } != FALSE {
                let token_handle = Handle::from(token_handle);
                entry.user = get_proc_user(*token_handle);
                entry.integrity_level = get_integrity_level(*token_handle);
            }

            // Grab a snapshot of the process' modules
            if let Some(h_snapshot_modules) = Handle::new(unsafe {
                CreateToolhelp32Snapshot(TH32CS_SNAPMODULE, pe_entry.th32ProcessID)
            }) {
                if unsafe { Module32First(*h_snapshot_modules, &mut mod_entry) } == TRUE {
                    // Add the bin path to the process listing information
                    entry.bin_path = unsafe {
                        Some(
                            std::ffi::CStr::from_ptr(mod_entry.szExePath.as_ptr())
                                .to_string_lossy()
                                .to_string(),
                        )
                    };
                }
            }
        }

        // Add the process information to the listing
        listing.push(entry);
    }

    let com_con = COMLibrary::new()?.into();
    let wmi_con = WMIConnection::new(com_con)?;

    #[derive(Deserialize, Debug)]
    #[serde(rename_all = "PascalCase")]
    #[serde(rename = "Win32_Process")]
    struct Win32ProcessCmdline {
        process_id: u32,
        command_line: Option<String>,
    }

    let results: Vec<Win32ProcessCmdline> = wmi_con.query()?;

    for result in results {
        for entry in listing.iter_mut() {
            if entry.process_id == result.process_id {
                entry.command_line = match result.command_line {
                    Some(cmd_line) => Some(cmd_line.to_owned()),
                    None => Some("".to_string()),
                };
                break;
            }
        }
    }

    // Return the process listing
    Ok(listing)
}

/// Grab the process name from the process entry
/// * `pe32` - winapi `PROCESSENTRY32` struct
fn get_proc_name(pe32: &PROCESSENTRY32) -> Option<String> {
    let name = unsafe {
        std::ffi::CStr::from_ptr(pe32.szExeFile.as_ptr())
            .to_string_lossy()
            .to_string()
    };

    Some(name)
}

/// Get the architecture of a process
/// * `handle` - Handle to the process
pub fn get_architecture(handle: *mut c_void) -> Option<String> {
    let mut is_wow64 = FALSE;

    // Query the process architecture
    if unsafe { IsWow64Process(handle, &mut is_wow64) } == FALSE {
        return None;
    }

    // Check if the process is 32 bit
    if is_wow64 == TRUE {
        Some("x86".to_string())
    } else {
        Some("x64".to_string())
    }
}

/// Get the user associated with a process
/// * `token` - Handle to the process token
pub fn get_proc_user(token: *mut c_void) -> Option<String> {
    let mut dw_len = 0;

    // Get the size for the process token information
    unsafe { GetTokenInformation(token, TokenUser, std::ptr::null_mut(), 0, &mut dw_len) };
    let mut buffer: Vec<u8> = vec![0; dw_len as usize];

    // Get the token information
    if unsafe {
        GetTokenInformation(
            token,
            TokenUser,
            buffer.as_mut_ptr() as *mut c_void,
            dw_len,
            &mut dw_len,
        )
    } == 0
    {
        return None;
    }

    let token_user: &TOKEN_USER = unsafe { &*buffer.as_ptr().cast() };

    // Grab the SID from the token
    let user_sid: PSID = token_user.User.Sid;
    let mut sid_type: SID_NAME_USE = Default::default();

    let mut lp_name = [0i8; 1024];
    let mut lp_domain = [0i8; 1024];

    // Lookup the SID
    if unsafe {
        LookupAccountSidA(
            std::ptr::null(),
            user_sid,
            lp_name.as_mut_ptr(),
            &mut dw_len,
            lp_domain.as_mut_ptr(),
            &mut dw_len,
            &mut sid_type,
        )
    } == FALSE
    {
        return None;
    }

    // Get the owner name and domain of the process
    let name = unsafe {
        std::ffi::CStr::from_ptr(lp_name.as_ptr())
            .to_string_lossy()
            .to_string()
    };
    let domain = unsafe {
        std::ffi::CStr::from_ptr(lp_domain.as_ptr())
            .to_string_lossy()
            .to_string()
    };

    Some(format!("{}\\{}", domain, name))
}

/// Get the integrity level of a process
/// * `token` - Process token
pub fn get_integrity_level(token: *mut c_void) -> Option<u32> {
    // Grab the token information length
    let mut len: u32 = 0;
    unsafe {
        GetTokenInformation(
            token,
            TokenIntegrityLevel,
            std::ptr::null_mut(),
            0,
            &mut len,
        )
    };
    if unsafe { GetLastError() } != 122 {
        return None;
    }

    let mut buffer: Vec<u8> = vec![0; len as usize];

    // Get the token information for the integrity level
    if unsafe {
        GetTokenInformation(
            token,
            TokenIntegrityLevel,
            buffer.as_mut_ptr() as *mut c_void,
            len,
            &mut len,
        )
    } == 0
    {
        return None;
    }

    // Get the integrity level from the token SID
    let p_til: &TOKEN_MANDATORY_LABEL = unsafe { &*buffer.as_ptr().cast() };

    let integrity_level_sid: &u32 = unsafe {
        let p_count = GetSidSubAuthorityCount(p_til.Label.Sid);
        if p_count.is_null() {
            return None;
        }
        let count = (*p_count) - 1;
        let integrity_level_ptr = GetSidSubAuthority(p_til.Label.Sid, count as u32);
        if integrity_level_ptr.is_null() {
            return None;
        }
        &*integrity_level_ptr
    };

    // Get the integrity level from the sid
    Some(integrity_level_sid >> 12)
}

/// Get the start time of a process
/// * `handle` - Handle to the process
pub fn get_start_time(handle: *mut c_void) -> Option<i64> {
    let mut creation_time: FILETIME = Default::default();
    let mut exit_time: FILETIME = Default::default();
    let mut kernel_time: FILETIME = Default::default();
    let mut user_time: FILETIME = Default::default();

    // Get the process time information
    if unsafe {
        GetProcessTimes(
            handle,
            &mut creation_time,
            &mut exit_time,
            &mut kernel_time,
            &mut user_time,
        )
    } == 0
    {
        return None;
    }

    // Grab the "windows epoch" of the time stamp
    let mut win_epoch =
        ((creation_time.dwHighDateTime as i64) << (4 * 8)) | creation_time.dwLowDateTime as i64;

    // Convert the "windows epoch" to a sane posix epoch
    win_epoch -= 11644473600000 * 10000;
    let posix_epoch = win_epoch / 10000000;

    // Convert the timestamp to local time
    let start_time = DateTime::from_timestamp(posix_epoch, 0)?;

    Some(start_time.timestamp_millis())
}
