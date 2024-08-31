use thanatos_protos::config::Config;

use crate::errors;

pub struct Agent {}

impl Agent {
    pub fn perform_checkin(agent_config: Config) -> Result<Agent, errors::ThanatosError> {
        Ok(Agent {})
    }

    pub fn run(self) {}
}
