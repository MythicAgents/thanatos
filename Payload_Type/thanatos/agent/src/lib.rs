use errors::ThanatosError;

use crate::agent::Agent;

mod agent;
pub mod config;
mod crypto;
mod errors;
mod logging;
mod native;
mod profiles;
mod utils;

pub fn entrypoint(agent_config: config::Config) {
    if let Err(e) = run_agent(agent_config) {
        debug!("Agent returned an error: {:?}", e);
    } else {
        debug!("Agent exited successfully");
    }
}

fn run_agent(agent_config: config::Config) -> Result<(), ThanatosError> {
    let agent_instance = Agent::new(&agent_config)?
        .initialize()
        .checkin(agent_config.eke)?;

    todo!();
}
