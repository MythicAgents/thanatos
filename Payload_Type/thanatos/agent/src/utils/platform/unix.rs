use std::{ffi::CStr, mem::MaybeUninit};

use crate::utils::{
    os::unix::{IntegrityLevelExt, PasswdEntry},
    platform::IntegrityLevel,
};

/// Grabs the platform information for Linux including the kernel version
/// and checks if the system has SELinux installed
pub fn detailed_name() -> Result<String, std::io::Error> {
    let mut buf = generic_name();

    let mut uname = MaybeUninit::<libc::utsname>::uninit();

    // Get the uname information from the system
    if unsafe { libc::uname(uname.as_mut_ptr()) } == -1 {
        return Err(std::io::Error::last_os_error());
    }

    let uname = unsafe { uname.assume_init() };
    let release_bytes = uname.release.map(|v| v as u8);
    let release = CStr::from_bytes_until_nul(release_bytes.as_slice())
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    buf.push(' ');
    buf.push_str(&release.to_string_lossy());

    if std::fs::exists("/sys/fs/selinux")? {
        buf.push_str(" (Security Enhanced)");
    }

    Ok(buf)
}

/// Grabs the generic platform name without the kernel version or SELinux
/// information
pub fn generic_name() -> String {
    "Linux".to_string()
}

/// Gets the username the agent is associated with
pub fn username() -> Result<String, std::io::Error> {
    let pwd = PasswdEntry::getpwuid(unsafe { libc::getuid() })?
        .ok_or_else(|| std::io::Error::from_raw_os_error(libc::ENOENT))?;

    Ok(pwd.name().to_string_lossy().to_string())
}

/// Grabs the hostname of the system
pub fn hostname() -> Result<String, std::io::Error> {
    let mut buf = [MaybeUninit::<i8>::zeroed(); 256];
    let buf = {
        let ret = unsafe { libc::gethostname(buf.as_mut_ptr().cast(), buf.len() - 1) };
        if ret == -1 {
            return Err(std::io::Error::last_os_error());
        }
        unsafe { buf.map(|ch| ch.assume_init()) }
    };

    Ok(unsafe { CStr::from_ptr(buf.as_ptr()) }
        .to_string_lossy()
        .to_string())
}

/// Grabs the domain name of the system
pub fn domain() -> Result<String, std::io::Error> {
    let mut buf = [MaybeUninit::<i8>::zeroed(); 256];
    let buf = {
        let ret = unsafe { libc::getdomainname(buf.as_mut_ptr().cast(), buf.len() - 1) };
        if ret == -1 {
            return Err(std::io::Error::last_os_error());
        }
        unsafe { buf.map(|ch| ch.assume_init()) }
    };

    Ok(unsafe { CStr::from_ptr(buf.as_ptr()) }
        .to_string_lossy()
        .to_string())
}

/// Grabs the process integrity level
pub fn integrity_level() -> Result<IntegrityLevel, std::io::Error> {
    Ok(IntegrityLevel::from_uid(unsafe { libc::getuid() }))
}
