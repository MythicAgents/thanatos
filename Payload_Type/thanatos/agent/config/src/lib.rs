//! Contains the configuration values for agent

cfg_if::cfg_if! {
    if #[cfg(feature = "full")] {
        use prost::Message;

        #[rustfmt::skip]
        mod config_pb;

        use errors::ThanatosError;
        use utils::uuid::Uuid;

        const CONFIG: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/config.bin"));

        pub struct ConfigVars(config_pb::Config);

        pub use config_pb::HttpConfig;

        impl ConfigVars {
            pub fn parse() -> Result<ConfigVars, ThanatosError> {
                Ok(ConfigVars(config_pb::Config::decode(CONFIG).map_err(|_| ThanatosError::ConfigParseError)?))
            }

            pub fn uuid(&self) -> Result<Uuid, ThanatosError> {
                Uuid::try_from(self.0.uuid.as_ref()).map_err(|_| ThanatosError::ConfigParseError)
            }

            pub fn working_hours_start(&self) -> u32 {
                self.0.working_hours_start
            }

            pub fn working_hours_end(&self) -> u32 {
                self.0.working_hours_end
            }

            pub fn connection_retries(&self) -> u32 {
                self.0.connection_retries
            }

            pub fn domains(&self) -> Result<Vec<[u8; 32]>, ThanatosError> {
                let domain_buffer = self.0.domains.as_ref().ok_or(ThanatosError::ConfigParseError)?;

                if domain_buffer.len() % 32 != 0 {
                    return Err(ThanatosError::ConfigParseError);
                }

                domain_buffer.chunks(32).map(|c| {
                    c[..32].try_into().map_err(|_| ThanatosError::ConfigParseError)
                }).collect::<Result<Vec<[u8; 32]>, ThanatosError>>()
            }

            pub fn hostnames(&self) -> Result<Vec<[u8; 32]>, ThanatosError> {
                let hostname_buffer = self.0.hostnames.as_ref().ok_or(ThanatosError::ConfigParseError)?;

                if hostname_buffer.len() % 32 != 0 {
                    return Err(ThanatosError::ConfigParseError);
                }

                hostname_buffer.chunks(32).map(|c| {
                    c[..32].try_into().map_err(|_| ThanatosError::ConfigParseError)
                }).collect::<Result<Vec<[u8; 32]>, ThanatosError>>()
            }

            pub fn usernames(&self) -> Result<Vec<[u8; 32]>, ThanatosError> {
                let username_buffer = self.0.usernames.as_ref().ok_or(ThanatosError::ConfigParseError)?;

                if username_buffer.len() % 32 != 0 {
                    return Err(ThanatosError::ConfigParseError);
                }

                username_buffer.chunks(32).map(|c| {
                    c[..32].try_into().map_err(|_| ThanatosError::ConfigParseError)
                }).collect::<Result<Vec<[u8; 32]>, ThanatosError>>()
            }

            pub fn spawn_to(&self) -> &str {
                self.0.spawn_to.as_ref()
            }

            pub fn http(&self) -> Option<&config_pb::HttpConfig> {
                self.0.http.as_ref()
            }
        }
    } else {
        #[rustfmt::skip]
        pub mod config_pb;
    }
}
