use std::time::SystemTime;

use chrono::{DateTime, NaiveTime, Utc};
use config::ConfigVars;
use errors::ThanatosError;

use super::workinghours;

pub struct Agent {
    uuid: String,
    working_hours: WorkingHours,
}

pub(super) struct WorkingHours {
    pub(super) start: NaiveTime,
    pub(super) end: NaiveTime,
}

impl Agent {
    pub fn new(config: ConfigVars) -> Result<Agent, ThanatosError> {
        Ok(Agent {
            uuid: config.uuid()?.to_string(),
            working_hours: WorkingHours {
                start: NaiveTime::from_num_seconds_from_midnight_opt(
                    config
                        .working_hours_start()
                        .try_into()
                        .map_err(|_| ThanatosError::ConfigParseError)?,
                    0,
                )
                .ok_or(ThanatosError::ConfigParseError)?,
                end: NaiveTime::from_num_seconds_from_midnight_opt(
                    config
                        .working_hours_end()
                        .try_into()
                        .map_err(|_| ThanatosError::ConfigParseError)?,
                    0,
                )
                .ok_or(ThanatosError::ConfigParseError)?,
            },
        })
    }

    pub fn run(self) {
        self.handle_working_hours();
    }

    fn handle_working_hours(&self) {
        if let Some(interval) = workinghours::calculate_working_hours(
            DateTime::<Utc>::from(SystemTime::now()),
            self.working_hours.start,
            self.working_hours.end,
        ) {
            std::thread::sleep(interval);
        }
    }
}
