//! This module is only imported when targeting windows
use serde::Serialize;
use std::ops::Deref;
use windows::Win32::Foundation::{CloseHandle, HANDLE};

pub mod whoami {
    use windows::core::PWSTR;
    use windows::Win32::Foundation::{ERROR_INSUFFICIENT_BUFFER, HANDLE};
    use windows::Win32::Security::{
        GetSidSubAuthority, GetSidSubAuthorityCount, GetTokenInformation, TokenIntegrityLevel,
        TOKEN_MANDATORY_LABEL, TOKEN_QUERY,
    };
    use windows::Win32::System::SystemInformation::{
        ComputerNameDnsDomain, ComputerNameDnsHostname, GetComputerNameExW,
    };
    use windows::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};
    use windows::Win32::System::WindowsProgramming::GetUserNameW;

    use super::Handle;
    use std::convert::TryInto;
    use std::os::windows::ffi::OsStringExt;

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
        let mut name_len = 0;
        // Call `GetUserNameW` to get the username length
        match unsafe { GetUserNameW(PWSTR::null(), &mut name_len) } {
            Err(e) if e == ERROR_INSUFFICIENT_BUFFER.into() => (),
            _ => return None,
        };

        let mut name: Vec<u16> = Vec::with_capacity(name_len.try_into().ok()?);
        name_len = name.capacity().try_into().ok()?;

        // Call `GetUserNameW` to get the current username
        unsafe {
            GetUserNameW(PWSTR(name.as_mut_ptr().cast()), &mut name_len).ok()?;
            name.set_len(name_len.try_into().ok()?);
        };

        // Remove the null-terminator
        name.pop();

        // Return the username as a String
        Some(
            std::ffi::OsString::from_wide(&name)
                .to_string_lossy()
                .to_string(),
        )
    }

    /// Get the hostname of the computer
    pub fn hostname() -> Option<String> {
        let mut name_len = 0;
        // Get the computer name length
        match unsafe { GetComputerNameExW(ComputerNameDnsHostname, PWSTR::null(), &mut name_len) } {
            Err(e) if e == ERROR_INSUFFICIENT_BUFFER.into() => (),
            _ => return None,
        };

        let mut name: Vec<u16> = Vec::with_capacity(name_len.try_into().ok()?);
        name_len = name.capacity().try_into().ok()?;

        // Get the computer hostname
        unsafe {
            GetComputerNameExW(
                ComputerNameDnsHostname,
                PWSTR(name.as_mut_ptr()),
                &mut name_len,
            )
            .ok()?;
            name.set_len(name_len.try_into().ok()?);
        };

        // Return the hostname as a String
        Some(
            std::ffi::OsString::from_wide(&name)
                .to_string_lossy()
                .to_string(),
        )
    }

    /// Get the domain name of the computer
    pub fn domain() -> Option<String> {
        let mut name_len = 0;
        // Get the domain name length
        let _ = unsafe { GetComputerNameExW(ComputerNameDnsDomain, PWSTR::null(), &mut name_len) };

        let mut name: Vec<u16> = Vec::with_capacity(name_len.try_into().ok()?);
        name_len = name.capacity().try_into().ok()?;

        // Get the domain name
        unsafe {
            GetComputerNameExW(
                ComputerNameDnsDomain,
                PWSTR(name.as_mut_ptr()),
                &mut name_len,
            )
            .ok()?;
            name.set_len(name_len.try_into().ok()?);
        };

        // Return the domain name as a String
        Some(
            std::ffi::OsString::from_wide(&name)
                .to_string_lossy()
                .to_string(),
        )
    }

    /// Get the integrity level
    pub fn get_integrity_level() -> Option<u32> {
        // Get a handle to the current process
        let p_handle = unsafe { GetCurrentProcess() };
        if p_handle.is_invalid() {
            return None;
        }

        // Grab the process' token
        let mut t_handle = HANDLE::default();
        unsafe { OpenProcessToken(p_handle, TOKEN_QUERY, &mut t_handle).ok()? };

        let t_handle = Handle::from(t_handle);

        // Grab the token information size
        let mut len: u32 = 0;
        match unsafe { GetTokenInformation(*t_handle, TokenIntegrityLevel, None, 0, &mut len) } {
            Err(e) if e == ERROR_INSUFFICIENT_BUFFER.into() => (),
            _ => return None,
        };

        let mut buffer: Vec<u8> = vec![0; len as usize];

        // Grab the current process token information
        unsafe {
            GetTokenInformation(
                *t_handle,
                TokenIntegrityLevel,
                Some(buffer.as_mut_ptr().cast()),
                len,
                &mut len,
            )
            .ok()?
        };

        // Get the integrity level from the token
        let integrity_level_sid: &u32 = unsafe {
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

            &*integrity_level_ptr.cast()
        };

        Some(integrity_level_sid >> 12)
    }

    #[cfg(test)]
    mod tests {
        #[test]
        fn whoami_debug() {
            let _ = super::username().unwrap();
        }
    }
}

/// Abstraction over windows handles for garbage collection
#[derive(Debug)]
pub struct Handle(HANDLE);

impl Drop for Handle {
    /// Close the handle when it goes out of scope
    fn drop(&mut self) {
        let _ = unsafe { CloseHandle(self.0) };
    }
}

impl Deref for Handle {
    type Target = HANDLE;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<usize> for Handle {
    /// Create a handle without checking if it's valid
    /// * `handle` - handle as a usize
    fn from(h: usize) -> Self {
        Handle(HANDLE(h as isize))
    }
}

impl From<HANDLE> for Handle {
    /// Create a handle without checking if it's valid
    /// * `handle` - handle as a raw pointer
    fn from(h: HANDLE) -> Self {
        Handle(h)
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
        ip: crate::utils::local_ipaddress::get().unwrap_or_else(|| "".to_string()),
        os: whoami::platform(),
        user: whoami::username().unwrap_or_else(|| "".to_string()),
        host: whoami::hostname().unwrap_or_else(|| "".to_string()),
        pid: std::process::id(),
        uuid: crate::payloadvars::payload_uuid(),
        architecture: std::env::consts::ARCH.to_string(),
        domain: whoami::domain(),
        integrity_level: whoami::get_integrity_level(),
    };

    serde_json::to_string(&info).unwrap()
}
