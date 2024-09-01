use std::time::Duration;

use windows::Win32::System::SystemInformation::GetSystemTime;

use crate::utils::consts::{HOUR_MULTIPLIER, MINUTE_MULTIPLIER};

pub fn get_timeofday() -> Duration {
    let current_time = unsafe { GetSystemTime() };

    Duration::from_secs(current_time.wHour as u64 * HOUR_MULTIPLIER)
        + Duration::from_secs(current_time.wMinute as u64 * MINUTE_MULTIPLIER)
}
