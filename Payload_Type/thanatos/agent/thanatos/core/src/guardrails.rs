use crate::log;

use crate::system;

use cryptolib::hash::Sha256;

use crate::errors::ThanatosError;

/*
#[inline(always)]
pub fn run_checks(agent_config: &config::Config) -> bool {
    if !run_check(agent_config.usernames(), system::username) {
        log!("Guardrail check failed for usernames");
        return false;
    }

    if !run_check(agent_config.hostnames(), system::hostname) {
        log!("Guardrail check failed for hostnames");
        return false;
    }

    if !run_check(agent_config.domains(), system::domain) {
        log!("Guardrail check failed for domain names");
        return false;
    }

    true
}

fn run_check<F>(list: &[u8], f: F) -> bool
where
    F: Fn() -> Result<String, ThanatosError>,
{
    if !list.is_empty() {
        let check_info = if let Ok(info) = f() {
            info
        } else {
            return false;
        };

        let mut h = Sha256::new();
        h.update(check_info.to_lowercase().as_bytes());
        let check_val = h.finalize();
        return list.chunks_exact(32).any(|v| v == check_val);
    }

    true
}
*/
