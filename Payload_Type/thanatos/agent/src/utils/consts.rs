use std::time::Duration;

/// Conversion factor for hours to seconds
pub const HOUR_MULTIPLIER: u64 = 3600;

/// Conversion factor for minutes to seconds
pub const MINUTE_MULTIPLIER: u64 = 60;

/// Maximum duration value for the working end time
pub const MAX_WORKING_END_DURATION: Duration = Duration::from_secs(24 * HOUR_MULTIPLIER);
