use std::time::Duration;

use crate::utils::consts::{HOUR_MULTIPLIER, MINUTE_MULTIPLIER};

use super::errors::ConfigParseError;

pub fn parse_working_hours(working_hours: &str) -> Result<Duration, ConfigParseError> {
    let mut s = working_hours.split(':');

    let hour: u64 = s
        .next()
        .map(|hour| hour.parse().ok())
        .flatten()
        .ok_or(ConfigParseError::WorkingHours)?;

    if hour > 23 {
        return Err(ConfigParseError::WorkingHours);
    }

    let minute: u64 = s
        .next()
        .map(|minute| minute.parse().ok())
        .flatten()
        .ok_or(ConfigParseError::WorkingHours)?;

    if minute > 59 {
        return Err(ConfigParseError::WorkingHours);
    }

    Ok(Duration::from_secs(hour * HOUR_MULTIPLIER)
        + Duration::from_secs(minute * MINUTE_MULTIPLIER))
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::parse_working_hours;

    #[test]
    fn good_values() {
        let cases = [
            ("00:00", 0),
            ("00:30", 1800),
            ("01:00", 3600),
            ("10:23", 37380),
            ("23:59", 86340),
        ];

        for (inp, expected) in cases {
            assert_eq!(
                parse_working_hours(inp).unwrap(),
                Duration::from_secs(expected)
            );
        }
    }

    #[test]
    fn bad_values() {
        let cases = ["", "23", "46", "Hello World", "24:20"];

        for case in cases {
            assert!(parse_working_hours(case).is_err());
        }
    }
}
