use base_profile::msg::checkin::{CheckinData, CheckinInfo};
use config::HttpConfig;

pub struct HttpC2Profile {
    uuid: String,
    _callback_host: String,
    _callback_port: u32,
}

impl HttpC2Profile {
    pub fn new(uuid: utils::uuid::Uuid, config: &HttpConfig) -> HttpC2Profile {
        HttpC2Profile {
            uuid: uuid.to_string(),
            _callback_host: config.callback_host.to_owned(),
            _callback_port: config.callback_port,
        }
    }

    pub fn uuid(&self) -> &str {
        &self.uuid
    }

    pub fn send_checkin(&mut self, data: CheckinData) {
        let _full_msg = CheckinInfo {
            uuid: self.uuid.clone(),
            data: Some(data),
        };
    }
}
