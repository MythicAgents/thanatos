#![forbid(unsafe_code)]

use config::ConfigVars;

mod guardrails;
pub mod logging;
pub mod native;

pub fn initialize_agent<'a, 'b: 'a, F>(f: F, config: &'a ConfigVars<'b>)
where
    F: Fn(&'a ConfigVars<'b>),
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

    f(config)
}
