use std::time::Duration;

use chrono::{DateTime, NaiveTime, TimeDelta, Utc};

pub fn calculate_working_hours(
    current_date: DateTime<Utc>,
    start: NaiveTime,
    end: NaiveTime,
) -> Option<Duration> {
    let current_time = current_date.time();

    if current_time < start {
        start.signed_duration_since(current_time).to_std().ok()
    } else if current_time >= end {
        (start.signed_duration_since(current_time) + TimeDelta::days(1))
            .to_std()
            .ok()
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use chrono::{DateTime, FixedOffset, NaiveTime, Utc};

    struct WorkingHoursTestCase {
        current_time: &'static str,
        start: &'static str,
        end: &'static str,
        interval: Option<Duration>,
    }

    impl WorkingHoursTestCase {
        fn current_date(&self) -> DateTime<Utc> {
            let time_string = format!("2024-01-01T{}-00:00", self.current_time,);

            let datetime = DateTime::<FixedOffset>::parse_from_rfc3339(&time_string)
                .expect("Failed to parse current_time");

            datetime.to_utc()
        }

        fn start(&self) -> NaiveTime {
            NaiveTime::parse_from_str(self.start, "%H:%M").expect("Failed to parse start time")
        }

        fn end(&self) -> NaiveTime {
            NaiveTime::parse_from_str(self.end, "%H:%M").expect("Failed to parse end time")
        }
    }

    #[test]
    fn current_time_in_range() {
        let test_case = WorkingHoursTestCase {
            current_time: "12:00:00", // 12pm
            start: "08:00",           // 8am
            end: "18:00",             // 6pm
            interval: None,           // Don't sleep
        };

        let res = super::calculate_working_hours(
            test_case.current_date(),
            test_case.start(),
            test_case.end(),
        );

        assert_eq!(res, test_case.interval);
    }

    #[test]
    fn current_time_before_start() {
        let test_case = WorkingHoursTestCase {
            current_time: "08:00:00",                     // 8am
            start: "09:00",                               // 9am
            end: "18:00",                                 // 6pm 18:00
            interval: Some(Duration::from_secs(60 * 60)), // Sleeps for 1 hour
        };

        let res = super::calculate_working_hours(
            test_case.current_date(),
            test_case.start(),
            test_case.end(),
        );

        assert_eq!(res, test_case.interval);
    }

    #[test]
    fn current_time_after_end() {
        let test_case = WorkingHoursTestCase {
            current_time: "23:00:00",                               // 11pm
            start: "08:00",                                         // 8am
            end: "18:00",                                           // 6pm 18:00
            interval: Some(Duration::from_secs((1 + 8) * 60 * 60)), // Sleeps for 9 hours
        };

        let res = super::calculate_working_hours(
            test_case.current_date(),
            test_case.start(),
            test_case.end(),
        );

        assert_eq!(res, test_case.interval);
    }

    #[test]
    fn current_time_is_start() {
        let test_case = WorkingHoursTestCase {
            current_time: "08:00:00", // 8am
            start: "08:00",           // 8am
            end: "18:00",             // 6pm 18:00
            interval: None,           // Don't sleep
        };

        let res = super::calculate_working_hours(
            test_case.current_date(),
            test_case.start(),
            test_case.end(),
        );

        assert_eq!(res, test_case.interval);
    }

    #[test]
    fn current_time_is_end() {
        let test_case = WorkingHoursTestCase {
            current_time: "18:00:00",                               // 6pm
            start: "08:00",                                         // 8am
            end: "18:00",                                           // 6pm 18:00
            interval: Some(Duration::from_secs((6 + 8) * 60 * 60)), // Sleeps for 14 hours
        };

        let res = super::calculate_working_hours(
            test_case.current_date(),
            test_case.start(),
            test_case.end(),
        );

        assert_eq!(res, test_case.interval);
    }
}
