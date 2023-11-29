//! Enumerates information about the current host. This includes all of the information for the
//! initial checkin.

use crate::errors::ThanatosError;
use std::ffi::CStr;
use std::mem::MaybeUninit;

use super::linhelpers::IfAddrs;

/// Get the current OS information including kernel version and whether SELinux is detected
pub fn platform() -> String {
    let kernel = "Linux".to_string();

    // Check if the system has SELinux.
    let selinux = if std::path::Path::new("/sys/fs/selinux").exists() {
        "(Security Enhanced)"
    } else {
        ""
    }
    .to_string();

    let mut name: MaybeUninit<libc::utsname> = MaybeUninit::uninit();

    // Get the uname information from the system
    if unsafe { libc::uname(name.as_mut_ptr()) } != 0 {
        if !selinux.is_empty() {
            return format!("{kernel} {selinux}");
        } else {
            return kernel;
        }
    }

    // Safe to `assume_init` here since errors are handled above.
    let name = unsafe { name.assume_init() };

    // Check if the `uname` libc call succeeded and return the platform along with
    // the kernel version; otherwise, just return the platform
    let release = if let Ok(release) = unsafe { CStr::from_ptr(name.release.as_ptr()) }.to_str() {
        release
    } else if !selinux.is_empty() {
        return format!("{kernel} {selinux}");
    } else {
        return kernel;
    };

    // Create the output including SELinux information
    if !selinux.is_empty() {
        format!("{kernel} {release} {selinux}")
    } else {
        format!("{kernel} {release}")
    }
}

/// Gets the current username the agent is running as.
pub fn username() -> Result<String, ThanatosError> {
    // Get the passwd entry for the current uid
    let passwd: &libc::passwd = unsafe {
        // Get the passwd struct from `getpwuid`
        let passwd = libc::getpwuid(libc::getuid());

        // Check if either `passwd` or the `pw_name` is NULL
        if passwd.is_null() || (*passwd).pw_name.is_null() {
            return Err(ThanatosError::os_error());
        }

        &*passwd.cast()
    };

    // Return the `pw_name` member of the `passwd` struct
    let name_str = unsafe { CStr::from_ptr(passwd.pw_name) };
    let u_name = name_str
        .to_str()
        .map_err(|_| ThanatosError::StringParseError)?;

    Ok(u_name.to_string())
}

/// Gets the hostname of the system.
pub fn hostname() -> Result<String, ThanatosError> {
    // Constant for the buffer length
    const NAME_LEN: usize = 256;

    let mut name: MaybeUninit<[i8; NAME_LEN]> = MaybeUninit::uninit();

    // Get the hostname from libc. Ensure that there is a NULL terminator by doing
    // `NAME_LEN - 1` so that there isn't an out of bounds read in the later `CStr::from_ptr`
    if unsafe { libc::gethostname(name.as_mut_ptr().cast(), NAME_LEN - 1) } == -1 {
        return Err(ThanatosError::os_error());
    }

    // Safe to `assume_init` since errors are checked above.
    let name = unsafe { name.assume_init() };

    // Name is guaranteed to have a NULL terminator since the buffer is a byte
    // larger than the size passed to `libc::gethostname`
    Ok(unsafe {
        CStr::from_ptr(name.as_ptr().cast())
            .to_str()
            .map_err(|_| ThanatosError::StringParseError)?
            .to_string()
    })
}

/// Returns the domain name for linux if any.
pub fn domain() -> Result<String, ThanatosError> {
    // Get the system domain name if it exists
    let name: &libc::utsname = unsafe {
        let name: *mut libc::utsname = std::ptr::null_mut();
        if libc::uname(name) != 0 {
            return Err(ThanatosError::os_error());
        }
        &*name.cast()
    };

    let domainname = unsafe { CStr::from_ptr(name.domainname.as_ptr()) }
        .to_str()
        .map_err(|_| ThanatosError::StringParseError)?;

    if domainname == "(none)" {
        return Ok("".to_string());
    }

    Ok(domainname.to_string())
}

/// Gets the integrity level.
/// Linux doesn't have the concept of integrity levels so just check if the effective
/// user id is 0.
pub fn integrity_level() -> u32 {
    // Check if we are root
    if unsafe { libc::geteuid() == 0 } {
        return 4;
    }

    return 2;
}

/// Gets the internal IP addresses for Linux.
pub fn internal_ips() -> Result<Vec<String>, ThanatosError> {
    Ok(IfAddrs::new()?
        .into_iter()
        .filter_map(|interface| {
            // Get the interfaces which have the BROADCAST, MULTICAST, UP and LOWER_UP
            // flags set. This is a reasonably good way of figuring out "useful"
            // interfaces being used. VPN interfaces will typically use POINTOPOINT in place
            // of the normal BROADCAST flag.
            (((interface.ifa_flags & libc::IFF_BROADCAST as u32) != 0
                || (interface.ifa_flags & libc::IFF_POINTOPOINT as u32) != 0)
                && (interface.ifa_flags & libc::IFF_MULTICAST as u32) != 0
                && (interface.ifa_flags & libc::IFF_UP as u32) != 0
                && (interface.ifa_flags & libc::IFF_LOWER_UP as u32) != 0
                && !interface.ifa_addr.is_null())
            .then(|| {
                let mut host: MaybeUninit<[u8; libc::NI_MAXHOST as usize]> = MaybeUninit::uninit();
                (unsafe {
                    libc::getnameinfo(
                        interface.ifa_addr,
                        std::mem::size_of::<libc::sockaddr_in>() as u32,
                        host.as_mut_ptr().cast(),
                        libc::NI_MAXHOST,
                        std::ptr::null_mut(),
                        0,
                        libc::NI_NUMERICHOST,
                    )
                } == 0)
                    .then(|| {
                        // Safe to assume init here since the error from `getnameinfo` is checked
                        // above.
                        CStr::from_bytes_until_nul(&unsafe { host.assume_init() })
                            .ok()
                            .map(|s| s.to_string_lossy().to_string())
                    })
                    .flatten()
            })
            .flatten()
        })
        .collect())
}

/// Gets the current process name
pub fn process_name() -> Result<String, ThanatosError> {
    std::fs::read_to_string("/proc/self/comm")
        .map_err(|_| ThanatosError::os_error())
        .map(|s| s.trim_end_matches("\n").to_string())
}
