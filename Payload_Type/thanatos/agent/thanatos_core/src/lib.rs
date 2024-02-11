#![forbid(unsafe_code)]

use config::{ConfigVars, InitOption};

mod guardrails;
pub mod logging;
pub mod native;

pub fn initialize_agent<F>(f: F, config: ConfigVars<'static>)
where
    F: Fn(ConfigVars<'static>) + Send + Sync + 'static,
{
    let domains = config.domains();
    if !domains.is_empty() && !guardrails::check_domain(domains) {
        return;
    }

    let hostnames = config.hostnames();
    if !hostnames.is_empty() && !guardrails::check_hostname(hostnames) {
        return;
    }

    let usernames = config.usernames();
    if !usernames.is_empty() && !guardrails::check_username(usernames) {
        return;
    }

    match config.init_option() {
        InitOption::Thread => {
            std::thread::spawn(move || f(config));
        }
        #[cfg(target_os = "linux")]
        InitOption::Fork => todo!(),
        InitOption::None => {
            f(config);
        }
    }
}
