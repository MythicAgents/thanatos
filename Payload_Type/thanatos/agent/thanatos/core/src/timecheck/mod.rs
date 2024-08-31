use chrono::{DateTime, NaiveTime, TimeDelta, Utc};

#[inline]
pub fn passed_killdate(killdate: DateTime<Utc>) -> bool {
    killdate >= Utc::now()
}

pub fn check_working_hours(start_time: TimeDelta, end_time: TimeDelta) -> Option<TimeDelta> {
    let current_tod = Utc::now().time().signed_duration_since(NaiveTime::MIN);

    if current_tod < start_time {
        Some(start_time - current_tod)
    } else if current_tod >= end_time {
        Some(TimeDelta::days(1) - current_tod + start_time)
    } else {
        None
    }
}
