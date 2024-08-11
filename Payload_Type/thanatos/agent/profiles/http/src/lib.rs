use thanatos_protos::config;

pub struct HttpC2Profile {}

impl HttpC2Profile {
    pub fn new(_agent_config: &config::Config) -> HttpC2Profile {
        HttpC2Profile {}
    }
}
