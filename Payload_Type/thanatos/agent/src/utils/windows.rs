//! This module is only imported when targeting windows
use serde::Serialize;

pub mod whoami {
    use std::{
        convert::TryInto,
        ffi::OsString,
        os::windows::{
            ffi::OsStringExt,
            io::{AsRawHandle, FromRawHandle, OwnedHandle},
        },
    };
    use windows::Win32::{
        Foundation::{HANDLE, INVALID_HANDLE_VALUE},
        Security::{
            GetSidSubAuthority, GetSidSubAuthorityCount, GetTokenInformation, TokenIntegrityLevel,
            TOKEN_MANDATORY_LABEL, TOKEN_QUERY,
        },
        System::{
            SystemInformation::{
                ComputerNameDnsDomain, ComputerNameDnsHostname, GetComputerNameExW,
            },
            Threading::{GetCurrentProcess, OpenProcessToken},
            WindowsProgramming::GetUserNameW,
        },
    };
    use windows_strings::PWSTR;

    /// Get the platform information
    /// TODO: Query the system to get the Windows version
    pub fn platform() -> String {
        "Windows".to_string()
    }

    /// Return the generic platform name (Windows)
    #[inline]
    pub fn generic_platform() -> String {
        platform()
    }

    /// Get the user the agent is associated with
    pub fn username() -> Option<String> {
        let mut len = 0;
        // Call `GetUserNameW` to get the username length
        unsafe {
            GetUserNameW(None, &mut len).ok()?;
        };
        if len == 0 {
            return None;
        }

        // Allocate memory for the username
        let mut name = Box::<[u16]>::new_uninit_slice(len.try_into().ok()?);

        // Call `GetUserNameW` to get the current username
        unsafe {
            GetUserNameW(Some(PWSTR(name.as_mut_ptr().cast())), &mut len).ok()?;
        };

        let name = unsafe { name.assume_init() };
        let len = usize::try_from(len).unwrap();

        // Return the username as a String
        Some(
            OsString::from_wide(&name[..len])
                .to_string_lossy()
                .to_string(),
        )
    }

    /// Get the hostname of the computer
    pub fn hostname() -> Option<String> {
        let mut len = 0;
        // Get the computer name length
        unsafe {
            GetComputerNameExW(ComputerNameDnsHostname, None, &mut len).ok()?;
        };
        if len == 0 {
            return None;
        }

        let mut name = Box::<[u16]>::new_uninit_slice(len.try_into().ok()?);

        // Get the computer hostname
        unsafe {
            GetComputerNameExW(
                ComputerNameDnsHostname,
                Some(PWSTR(name.as_mut_ptr().cast())),
                &mut len,
            )
            .ok()?;
        };

        let name = unsafe { name.assume_init() };
        let len = usize::try_from(len).unwrap();

        // Return the hostname as a String
        Some(
            std::ffi::OsString::from_wide(&name[..len])
                .to_string_lossy()
                .to_string(),
        )
    }

    /// Get the domain name of the computer
    pub fn domain() -> Option<String> {
        let mut len = 0;
        // Get the domain name length
        unsafe {
            GetComputerNameExW(ComputerNameDnsDomain, None, &mut len).ok()?;
        };
        if len == 0 {
            return None;
        }

        let mut name = Box::<[u16]>::new_uninit_slice(len.try_into().ok()?);

        // Get the domain name
        unsafe {
            GetComputerNameExW(
                ComputerNameDnsDomain,
                Some(PWSTR(name.as_mut_ptr().cast())),
                &mut len,
            )
            .ok()?;
        };

        let name = unsafe { name.assume_init() };
        let len = usize::try_from(len).unwrap();

        // Return the domain name as a String
        Some(
            std::ffi::OsString::from_wide(&name[..len])
                .to_string_lossy()
                .to_string(),
        )
    }

    /// Get the integrity level
    pub fn get_integrity_level() -> Option<u32> {
        // Grab the process' token
        let thread_handle = unsafe { OwnedHandle::from_raw_handle(INVALID_HANDLE_VALUE.0) };
        unsafe {
            OpenProcessToken(
                GetCurrentProcess(),
                TOKEN_QUERY,
                &mut HANDLE(thread_handle.as_raw_handle()),
            )
            .ok()?
        };

        // Grab the token information size
        let mut len = 0u32;
        let _ = unsafe {
            GetTokenInformation(
                HANDLE(thread_handle.as_raw_handle()),
                TokenIntegrityLevel,
                None,
                0,
                &mut len,
            )
        };
        if len == 0 {
            return None;
        }

        let mut buffer = Box::<[u8]>::new_uninit_slice(len.try_into().ok()?);

        // Grab the current process token information
        unsafe {
            GetTokenInformation(
                HANDLE(thread_handle.as_raw_handle()),
                TokenIntegrityLevel,
                Some(buffer.as_mut_ptr().cast()),
                len,
                &mut len,
            )
            .ok()?;
        }

        let buffer = unsafe { buffer.assume_init() };

        // Get the integrity level from the token
        let integrity_level_sid: u32 = unsafe {
            let til: &TOKEN_MANDATORY_LABEL = &*buffer.as_ptr().cast();
            let p_count = GetSidSubAuthorityCount(til.Label.Sid);
            if p_count.is_null() {
                return None;
            }
            let count = (*p_count) - 1;
            let integrity_level_ptr = GetSidSubAuthority(til.Label.Sid, count as u32);
            if integrity_level_ptr.is_null() {
                return None;
            }

            *integrity_level_ptr.cast()
        };

        Some(integrity_level_sid >> 12)
    }
}

/// Initial check in information for Windows
#[derive(Serialize)]
struct CheckinInfo {
    /// Action (checkin)
    action: String,

    /// Internal IP address
    ip: String,

    /// OS of the system
    os: String,

    /// Username associated with the callback
    user: String,

    /// Hostname of the system
    host: String,

    /// Process id of the agent
    pid: u32,

    /// Mythic UUID
    uuid: String,

    /// architecture of the agent
    architecture: String,

    /// Domain name of the system
    domain: Option<String>,

    /// Integrity level of the agent
    integrity_level: Option<u32>,
}

/// Get the checkin information for windows
pub fn get_checkin_info() -> String {
    // Grab the initial checkin information for Windows hosts
    let info = CheckinInfo {
        action: "checkin".to_string(),
        ip: crate::utils::local_ipaddress::get().unwrap_or_default(),
        os: whoami::platform(),
        user: whoami::username().unwrap_or_default(),
        host: whoami::hostname().unwrap_or_default(),
        pid: std::process::id(),
        uuid: crate::payloadvars::payload_uuid(),
        architecture: std::env::consts::ARCH.to_string(),
        domain: whoami::domain(),
        integrity_level: whoami::get_integrity_level(),
    };

    serde_json::to_string(&info).unwrap()
}
