use thanatos_protos::{config::HttpConfig, msg::MythicResponse};

use crate::errors::ProfileError;

pub struct HttpC2Profile {}

impl HttpC2Profile {
    pub fn new(config: &HttpConfig) -> HttpC2Profile {
        Self {}
    }

    pub fn send_data(data: &[u8]) -> Result<MythicResponse, ProfileError> {
        todo!();
    }
}
