//! This module is only imported when targeting Windows hosts
use crate::ps::ProcessListingEntry;
use chrono::DateTime;
use serde::Deserialize;
use std::os::windows::io::{AsHandle, AsRawHandle, BorrowedHandle, FromRawHandle};
use std::result::Result;
use std::{error::Error, os::windows::io::OwnedHandle};
use windows::Win32::Foundation::{
    ERROR_NO_MORE_FILES, FALSE, FILETIME, HANDLE, INVALID_HANDLE_VALUE, TRUE,
};
use windows::Win32::Security::{
    GetSidSubAuthority, GetSidSubAuthorityCount, GetTokenInformation, LookupAccountSidA,
    TokenIntegrityLevel, TokenUser, SID_NAME_USE, TOKEN_MANDATORY_LABEL, TOKEN_QUERY, TOKEN_USER,
};
use windows::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, Module32First, Process32First, Process32Next, MODULEENTRY32,
    PROCESSENTRY32, TH32CS_SNAPMODULE, TH32CS_SNAPPROCESS,
};
use windows::Win32::System::Threading::{
    GetProcessTimes, IsWow64Process, OpenProcess, OpenProcessToken, PROCESS_QUERY_INFORMATION,
};
use windows_strings::PSTR;
use wmi::WMIConnection;

/// Get the list of process information
pub fn process_info() -> Result<Vec<ProcessListingEntry>, Box<dyn Error>> {
    // Vec to hold the process infomation
    let mut listing: Vec<ProcessListingEntry> = Vec::new();

    // Grab a snapshot of all the running processes
    let snapshot =
        unsafe { OwnedHandle::from_raw_handle(CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0)?.0) };

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
    if unsafe { Process32First(HANDLE(snapshot.as_raw_handle()), &mut pe_entry).is_ok() } {
        // Query the process token
        if let Ok(process) = unsafe {
            OpenProcess(PROCESS_QUERY_INFORMATION, false, pe_entry.th32ProcessID)
                .map(|handle| OwnedHandle::from_raw_handle(handle.0))
        } {
            // Populate the `ProcessListingEntry` with the process information
            first_entry.architecture = get_architecture(process.as_handle()).unwrap_or_default();
            first_entry.name = get_proc_name(&pe_entry);
            first_entry.start_time = get_start_time(process.as_handle());

            let token_handle = unsafe { OwnedHandle::from_raw_handle(INVALID_HANDLE_VALUE.0) };
            if unsafe {
                OpenProcessToken(
                    HANDLE(process.as_raw_handle()),
                    TOKEN_QUERY,
                    &mut HANDLE(token_handle.as_raw_handle()),
                )
                .is_ok()
            } {
                first_entry.user = get_proc_user(token_handle.as_handle());
                first_entry.integrity_level = get_integrity_level(token_handle.as_handle());
            }
        }
        // Grab a snapshot of the process' modules
        if let Ok(modules) = unsafe {
            CreateToolhelp32Snapshot(TH32CS_SNAPMODULE, pe_entry.th32ProcessID)
                .map(|handle| OwnedHandle::from_raw_handle(handle.0))
        } {
            if unsafe { Module32First(HANDLE(modules.as_raw_handle()), &mut mod_entry).is_ok() } {
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
        match unsafe { Process32Next(HANDLE(snapshot.as_raw_handle()), &mut pe_entry) } {
            Err(e) if e.code() == ERROR_NO_MORE_FILES.to_hresult() => {
                break;
            }
            Err(_) => continue,
            _ => (),
        }

        // Create the listing entry struct
        let mut entry = ProcessListingEntry {
            process_id: pe_entry.th32ProcessID,
            architecture: "".to_string(),
            parent_process_id: Some(pe_entry.th32ParentProcessID),
            ..Default::default()
        };

        // Open a handle to the process token
        if let Ok(handle) = unsafe {
            OpenProcess(PROCESS_QUERY_INFORMATION, true, pe_entry.th32ProcessID)
                .map(|handle| OwnedHandle::from_raw_handle(handle.0))
        } {
            entry.architecture = get_architecture(handle.as_handle()).unwrap_or_default();
            entry.name = get_proc_name(&pe_entry);
            entry.start_time = get_start_time(handle.as_handle());

            let token_handle = unsafe { OwnedHandle::from_raw_handle(INVALID_HANDLE_VALUE.0) };
            if unsafe {
                OpenProcessToken(
                    HANDLE(handle.as_raw_handle()),
                    TOKEN_QUERY,
                    &mut HANDLE(token_handle.as_raw_handle()),
                )
                .is_ok()
            } {
                entry.user = get_proc_user(token_handle.as_handle());
                entry.integrity_level = get_integrity_level(token_handle.as_handle());
            }

            // Grab a snapshot of the process' modules
            if let Ok(modules) = unsafe {
                CreateToolhelp32Snapshot(TH32CS_SNAPMODULE, pe_entry.th32ProcessID)
                    .map(|handle| OwnedHandle::from_raw_handle(handle.0))
            } {
                if unsafe { Module32First(HANDLE(modules.as_raw_handle()), &mut mod_entry).is_ok() }
                {
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

    let wmi_con = WMIConnection::new()?;

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
pub fn get_architecture(handle: BorrowedHandle) -> Option<String> {
    let mut is_wow64 = FALSE;

    // Query the process architecture
    unsafe { IsWow64Process(HANDLE(handle.as_raw_handle()), &mut is_wow64).ok()? };

    // Check if the process is 32 bit
    if is_wow64 == TRUE {
        Some("x86".to_string())
    } else {
        Some("x64".to_string())
    }
}

/// Get the user associated with a process
/// * `token` - Handle to the process token
pub fn get_proc_user(token: BorrowedHandle) -> Option<String> {
    let mut len = 0;

    // Get the size for the process token information
    let _ =
        unsafe { GetTokenInformation(HANDLE(token.as_raw_handle()), TokenUser, None, 0, &mut len) };
    if len == 0 {
        return None;
    }
    let mut buffer = Box::<[u8]>::new_uninit_slice(len as usize);

    // Get the token information
    unsafe {
        GetTokenInformation(
            HANDLE(token.as_raw_handle()),
            TokenUser,
            Some(buffer.as_mut_ptr().cast()),
            len,
            &mut len,
        )
        .ok()?;
    }

    let buffer = unsafe { buffer.assume_init() };
    let token_user: &TOKEN_USER = unsafe { &*buffer.as_ptr().cast() };

    // Grab the SID from the token
    let user_sid = token_user.User.Sid;
    let mut sid_type = SID_NAME_USE::default();

    let mut lp_name = [0i8; 1024];
    let mut lp_domain = [0i8; 1024];

    // Lookup the SID
    unsafe {
        LookupAccountSidA(
            None,
            user_sid,
            Some(PSTR(lp_name.as_mut_ptr().cast())),
            &mut len,
            Some(PSTR(lp_domain.as_mut_ptr().cast())),
            &mut len,
            &mut sid_type,
        )
        .ok()?;
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
pub fn get_integrity_level(token: BorrowedHandle) -> Option<u32> {
    // Grab the token information length
    let mut len: u32 = 0;
    let _ = unsafe {
        GetTokenInformation(
            HANDLE(token.as_raw_handle()),
            TokenIntegrityLevel,
            None,
            0,
            &mut len,
        )
    };
    if len == 0 {
        return None;
    }

    let mut buffer = Box::<[u8]>::new_uninit_slice(len as usize);

    // Get the token information for the integrity level
    unsafe {
        GetTokenInformation(
            HANDLE(token.as_raw_handle()),
            TokenIntegrityLevel,
            Some(buffer.as_mut_ptr().cast()),
            len,
            &mut len,
        )
        .ok()?;
    }

    let buffer = unsafe { buffer.assume_init() };

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
pub fn get_start_time(handle: BorrowedHandle) -> Option<i64> {
    let mut creation_time = FILETIME::default();
    let mut exit_time = FILETIME::default();
    let mut kernel_time = FILETIME::default();
    let mut user_time = FILETIME::default();

    // Get the process time information
    unsafe {
        GetProcessTimes(
            HANDLE(handle.as_raw_handle()),
            &mut creation_time,
            &mut exit_time,
            &mut kernel_time,
            &mut user_time,
        )
        .ok()?;
    };

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
