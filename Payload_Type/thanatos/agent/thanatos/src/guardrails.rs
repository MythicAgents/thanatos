use crate::debug;

use crate::system;

use cryptolib::hash::Sha256;

use errors::ThanatosError;
use thanatos_protos::config;

#[inline(always)]
pub fn run_checks(agent_config: &config::Config) -> bool {
    if !run_check(agent_config.usernames(), system::username) {
        debug!("Guardrail check failed for usernames");
        return false;
    }

    if !run_check(agent_config.hostnames(), system::hostname) {
        debug!("Guardrail check failed for hostnames");
        return false;
    }

    if !run_check(agent_config.domains(), system::domain) {
        debug!("Guardrail check failed for domain names");
        return false;
    }

    true
}

fn run_check<F>(list: &[u8], f: F) -> bool
where
    F: Fn() -> Result<String, ThanatosError>,
{
    if !list.is_empty() {
        let check_val = match f().map(|v| {
            let mut h = Sha256::new();
            h.update(v.to_lowercase().as_bytes());
            h.finalize()
        }) {
            Ok(v) => v,
            Err(_) => return false,
        };

        return list.chunks_exact(32).any(|v| v == &check_val);
    }

    true
}
