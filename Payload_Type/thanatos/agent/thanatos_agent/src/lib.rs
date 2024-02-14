#![forbid(unsafe_code)]

use config::ConfigVars;

#[cfg(any(
    feature = "usernamecheck",
    feature = "hostnamecheck",
    feature = "domaincheck",
    test
))]
mod guardrails;

pub mod logging;
pub mod native;

pub fn entrypoint() {
    let agent_config = if let Ok(c) = ConfigVars::parse() {
        c
    } else {
        return;
    };

    #[cfg(feature = "usernamecheck")]
    if let Ok(usernames) = agent_config.usernames() {
        if !guardrails::check_username(&usernames) {
            return;
        }
    } else {
        return;
    }

    #[cfg(feature = "hostnamecheck")]
    if let Ok(hostnames) = agent_config.hostnames() {
        if !guardrails::check_hostname(&hostnames) {
            return;
        }
    } else {
        return;
    }

    #[cfg(feature = "domaincheck")]
    if let Ok(domains) = agent_config.domains() {
        if !guardrails::check_domain(&domains) {
            return;
        }
    } else {
        return;
    }

    #[cfg(feature = "init-thread")]
    std::thread::spawn(|| run_agent(agent_config));

    #[cfg(all(target_os = "linux", feature = "init-fork"))]
    native::linux::fork(|| run_agent(agent_config));

    #[cfg(not(any(feature = "init-thread", feature = "init-fork")))]
    run_agent(agent_config);
}

fn run_agent(_agent_config: ConfigVars) {}
