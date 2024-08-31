//! This module is only imported when targeting windows
use serde::Serialize;
use std::ops::Deref;
use winapi::{
    ctypes::c_void,
    um::handleapi::{CloseHandle, INVALID_HANDLE_VALUE},
};

pub mod whoami {
    use super::Handle;
    use std::convert::TryInto;
    use std::os::raw::c_ulong;
    use std::os::windows::ffi::OsStringExt;
    use winapi::{
        ctypes::c_void,
        um::{
            processthreadsapi::{GetCurrentProcess, OpenProcessToken},
            securitybaseapi::{GetSidSubAuthority, GetSidSubAuthorityCount, GetTokenInformation},
            sysinfoapi::{ComputerNameDnsDomain, ComputerNameDnsHostname, GetComputerNameExW},
            winbase::GetUserNameW,
            winnt::{TokenIntegrityLevel, TOKEN_MANDATORY_LABEL, TOKEN_QUERY},
        },
    };

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
        let _ = unsafe {
            GetUserNameW(std::ptr::null_mut(), &mut name_len);
        };

        let mut name: Vec<u16> = Vec::with_capacity(name_len.try_into().unwrap_or(usize::MAX));
        name_len = name.capacity().try_into().unwrap_or(c_ulong::MAX);

        // Call `GetUserNameW` to get the current username
        let ret = unsafe {
            let ret = GetUserNameW(name.as_mut_ptr().cast(), &mut name_len);
            name.set_len(name_len.try_into().unwrap_or(usize::MAX));
            ret
        };

        if ret == 0 {
            return None;
        }

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
        let _ = unsafe {
            GetComputerNameExW(ComputerNameDnsHostname, std::ptr::null_mut(), &mut name_len);
        };

        let mut name: Vec<u16> = Vec::with_capacity(name_len.try_into().unwrap_or(usize::MAX));
        name_len = name.capacity().try_into().unwrap_or(c_ulong::MAX);

        // Get the computer hostname
        let ret = unsafe {
            let ret = GetComputerNameExW(ComputerNameDnsHostname, name.as_mut_ptr(), &mut name_len);
            name.set_len(name_len.try_into().unwrap_or(usize::MAX));
            ret
        };

        if ret == 0 {
            return None;
        }

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
        let _ = unsafe {
            GetComputerNameExW(ComputerNameDnsDomain, std::ptr::null_mut(), &mut name_len)
        };

        let mut name: Vec<u16> = Vec::with_capacity(name_len.try_into().unwrap_or(usize::MAX));
        name_len = name.capacity().try_into().unwrap_or(c_ulong::MAX);

        // Get the domain name
        let ret = unsafe {
            let ret = GetComputerNameExW(ComputerNameDnsDomain, name.as_mut_ptr(), &mut name_len);
            name.set_len(name_len.try_into().unwrap_or(usize::MAX));
            ret
        };

        if ret == 0 {
            return None;
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
        if p_handle.is_null() {
            return None;
        }

        // Grab the process' token
        let mut t_handle: *mut c_void = std::ptr::null_mut();
        if unsafe { OpenProcessToken(p_handle, TOKEN_QUERY, &mut t_handle) == 0 } {
            return None;
        }

        let t_handle = Handle::new(t_handle)?;

        // Grab the token information size
        let mut len: u32 = 0;
        if unsafe {
            GetTokenInformation(
                *t_handle,
                TokenIntegrityLevel,
                std::ptr::null_mut(),
                0,
                &mut len,
            ) == 1
        } {
            return None;
        }

        let mut buffer: Vec<u8> = vec![0; len as usize];

        // Grab the current process token information
        if unsafe {
            GetTokenInformation(
                *t_handle,
                TokenIntegrityLevel,
                buffer.as_mut_ptr() as *mut c_void,
                len,
                &mut len,
            ) == 0
        } {
            return None;
        }

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
}

/// Abstraction over windows handles for garbage collection
#[derive(Debug)]
pub struct Handle(*mut c_void);

impl Drop for Handle {
    /// Close the handle when it goes out of scope
    fn drop(&mut self) {
        unsafe { CloseHandle(self.0) };
    }
}

impl Deref for Handle {
    type Target = *mut c_void;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Handle {
    /// Create a new handle from a raw pointer
    /// Returns `None` if the handle is invalid
    /// * `handle` - handle pointer
    pub fn new(h: *mut c_void) -> Option<Self> {
        if h.is_null() || h == INVALID_HANDLE_VALUE {
            None
        } else {
            Some(Handle(h))
        }
    }
}

impl From<usize> for Handle {
    /// Create a handle without checking if it's valid
    /// * `handle` - handle as a usize
    fn from(h: usize) -> Self {
        Handle(h as *mut c_void)
    }
}

impl From<*mut c_void> for Handle {
    /// Create a handle without checking if it's valid
    /// * `handle` - handle as a raw pointer
    fn from(h: *mut c_void) -> Self {
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
