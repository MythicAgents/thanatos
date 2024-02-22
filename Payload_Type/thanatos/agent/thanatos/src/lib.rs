#![forbid(unsafe_code)]

use agent::Agent;
use config::ConfigVars;

mod agent;
mod guardrails;
mod logging;
mod native;
mod os;
mod proto;

pub fn entrypoint() {
    let agent_config = if let Ok(c) = ConfigVars::parse() {
        c
    } else {
        return;
    };

    if !guardrails::run_guardrails(&agent_config) {
        return;
    }

    #[cfg(feature = "init-thread")]
    std::thread::spawn(|| run_agent(agent_config));

    #[cfg(feature = "init-fork")]
    todo!();

    #[cfg(not(any(feature = "init-thread", feature = "init-fork")))]
    run_agent(agent_config);
}

fn run_agent(agent_config: ConfigVars) {
    if let Ok(agent_instance) = Agent::new(agent_config) {
        agent_instance.run();
    }
}
