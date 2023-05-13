//! This module is only imported when targeting Linux
use serde::Serialize;
use std::ffi::CStr;

pub mod whoami {
    use std::ffi::CStr;

    /// Grabs the platform information for Linux including the kernel version
    /// and checks if the system has SELinux installed
    pub fn platform() -> String {
        let mut name: libc::utsname = unsafe { std::mem::zeroed() };

        let kernel = "Linux".to_string();

        // Check if the system is SELinux
        let selinux = if std::path::Path::new("/sys/fs/selinux").exists() {
            "(Security Enhanced)"
        } else {
            ""
        }
        .to_string();

        // Get the uname information from the system
        if unsafe { libc::uname(&mut name) } != 0 {
            if !selinux.is_empty() {
                return format!("{} {}", kernel, selinux);
            } else {
                return kernel;
            }
        }

        // Check if the `uname` libc call succeeded and return the platform along with
        // the kernel version; otherwise, just return the platform
        let release = if let Ok(release) = unsafe { CStr::from_ptr(name.release.as_ptr()) }.to_str()
        {
            release
        } else if !selinux.is_empty() {
            return format!("{} {}", kernel, selinux);
        } else {
            return kernel;
        };

        // Create the output including SELinux information
        if !selinux.is_empty() {
            format!("{} {} {}", kernel, release, selinux)
        } else {
            format!("{} {}", kernel, release)
        }
    }

    /// Grabs the generic platform name without the kernel version or SELinux
    /// information
    pub fn generic_platform() -> String {
        "Linux".to_string()
    }

    /// Gets the username the agent is associated with
    pub fn username() -> Option<String> {
        // Get the passwd entry for the current uid
        let passwd: &libc::passwd = unsafe {
            let passwd = libc::getpwuid(libc::getuid());
            if passwd.is_null() || (*passwd).pw_name.is_null() {
                return None;
            }

            &*passwd.cast()
        };

        // Return the `pw_name` member of the `passwd` struct
        let name_str = unsafe { CStr::from_ptr(passwd.pw_name) };
        let u_name = name_str.to_str().unwrap().to_owned();

        Some(u_name)
    }

    /// Grabs the hostname of the system
    pub fn hostname() -> Option<String> {
        let mut host = [0i8; 256];

        // Get the system hostname using libc
        let name_ptr = unsafe {
            let ret = libc::gethostname(host.as_mut_ptr(), 255);
            if ret == -1 {
                return None;
            }
            CStr::from_ptr(host.as_ptr())
        };

        Some(name_ptr.to_str().unwrap().to_owned())
    }

    /// Grabs the domain name of the system
    pub fn domain() -> Option<String> {
        // Get the system domain name if it exists
        let name: &libc::utsname = unsafe {
            let name: *mut libc::utsname = std::ptr::null_mut();
            if libc::uname(name) != 0 {
                return None;
            }
            &*name.cast()
        };

        let domainname = unsafe { CStr::from_ptr(name.domainname.as_ptr()) }
            .to_str()
            .ok()?;

        if domainname == "(none)" {
            return None;
        }

        Some(domainname.to_string())
    }
}

/// Converts an integer uid to its corresponding user name
/// * `uid` - UID for the user to get the username from
pub fn get_user_from_uid(uid: u32) -> Option<String> {
    // Get the passwd entry for the uid parameter
    let pw_struct = unsafe { libc::getpwuid(uid) };
    if pw_struct.is_null() {
        return None;
    }

    // Return the username as a String
    let raw_name = unsafe { CStr::from_ptr((*pw_struct).pw_name) };
    raw_name.to_str().map(|x| x.to_string()).ok()
}

/// Converts an integer gid to its corresponding group name
pub fn get_group_from_gid(gid: u32) -> Option<String> {
    // Get the groupd file entry
    let g_struct = unsafe { libc::getgrgid(gid) };
    if g_struct.is_null() {
        return None;
    }

    // Return the group name as a String
    let raw_group = unsafe { CStr::from_ptr((*g_struct).gr_name) };
    raw_group.to_str().map(|x| x.to_string()).ok()
}

/// Checkin info for Mythic initial check in
#[derive(Serialize)]
struct CheckinInfo {
    /// Action (checkin)
    action: String,

    /// Internal IP address
    ips: Vec<String>,

    /// OS information
    os: String,

    /// User name
    user: String,

    /// Host name
    host: String,

    /// Current process id
    pid: u32,

    /// Mythic UUID
    uuid: String,

    /// Agent architecture
    architecture: String,

    /// Agent integrity level
    integrity_level: u32,

    /// Machine domain name
    domain: Option<String>,
}

/// Get the check in information for linux systems
pub fn get_checkin_info() -> String {
    // Get the current uid
    let uid = unsafe { libc::getuid() };

    // Set the integrity level to 3 if running as root
    let integrity_level = if uid == 0 { 3 } else { 2 };

    let info = CheckinInfo {
        action: "checkin".to_string(),
        ips: vec![crate::utils::local_ipaddress::get().unwrap_or_else(|| "".to_string())],
        os: whoami::platform(),
        user: whoami::username().unwrap_or_else(|| "".to_string()),
        host: whoami::hostname().unwrap_or_else(|| "".to_string()),
        pid: std::process::id(),
        uuid: crate::payloadvars::payload_uuid(),
        architecture: std::env::consts::ARCH.to_string(),
        integrity_level,
        domain: whoami::domain(),
    };

    serde_json::to_string(&info).unwrap()
}
