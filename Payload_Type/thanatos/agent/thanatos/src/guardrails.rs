#![allow(unused)]

#[cfg(target_os = "linux")]
use crate::os::linux as system;

#[cfg(target_os = "windows")]
use crate::os::windows as system;

use cryptolib::hash::Sha256;

use errors::ThanatosError;

#[inline(always)]
pub fn run_guardrails() -> bool {
    if !run_check(agent_config.usernames(), system::username) {
        return false;
    }

    if !run_check(agent_config.hostnames(), system::hostname) {
        return false;
    }

    if !run_check(agent_config.domains(), system::domain) {
        return false;
    }

    true
}

fn run_check<F>(list: Result<Vec<[u8; 32]>, ThanatosError>, f: F) -> bool
where
    F: Fn() -> Result<String, ThanatosError>,
{
    let list = if let Ok(ref l) = list {
        l.as_slice()
    } else {
        return false;
    };

    let val = if let Ok(v) = f() { v } else { return false };
    check_hashlist_with(list, &val)
}

fn check_hashlist_with(hlist: &[[u8; 32]], value: &str) -> bool {
    let value = value.to_lowercase();

    let mut h = Sha256::new();
    h.update(value.as_bytes());
    let result = h.finalize();
    hlist.iter().any(|v| v == &result)
}

#[cfg(test)]
mod tests {
    use super::{check_hashlist_with, run_check};
    use cryptolib::hash::Sha256;
    use errors::ThanatosError;

    #[test]
    fn matched_check() {
        let input_value = "my.spooky.domain";
        let hlist_with_value = vec![hex_literal::hex!(
            "6b0a38edbe6d724b1679bf3ba6ed862975b2403019c7a95f8257d4e840d60df1"
        )];

        assert!(check_hashlist_with(&hlist_with_value, input_value));
    }

    #[test]
    fn check_nomatch() {
        let input_value = "my.spooky.domain";
        let hlist_without_value = vec![hex_literal::hex!(
            // garbage
            "795b6904e54f82411df4b0e27a373a55eea3f9d66dac5a9bce1dd92f7b401da5"
        )];

        assert!(!check_hashlist_with(&hlist_without_value, input_value));
    }

    #[test]
    fn mixed_casing() {
        let input_value = "my.SPOOKY.domain";
        let hlist = vec![hex_literal::hex!(
            // my.spooky.domain
            "6b0a38edbe6d724b1679bf3ba6ed862975b2403019c7a95f8257d4e840d60df1"
        )];

        assert!(check_hashlist_with(&hlist, input_value));
    }

    #[test]
    fn empty_value() {
        let hlist = vec![hex_literal::hex!(
            // garbage
            "795b6904e54f82411df4b0e27a373a55eea3f9d66dac5a9bce1dd92f7b401da5"
        )];

        assert!(!check_hashlist_with(&hlist, ""));
    }

    #[test]
    fn empty_domain_list() {
        assert!(!check_hashlist_with(&Vec::new(), "foo"));
    }

    #[test]
    fn system_hostname_test() {
        let host = super::system::hostname().expect("Failed to get hostname");
        let mut h = Sha256::new();
        h.update(host.as_bytes());
        let hash_val = h.finalize();

        let hash_list: Result<Vec<[u8; 32]>, ThanatosError> = Ok(vec![hash_val]);
        assert!(run_check(hash_list, super::system::hostname));
    }

    #[test]
    fn system_domain_test() {
        let domain = super::system::domain().expect("Failed to get domain");
        let mut h = Sha256::new();
        h.update(domain.as_bytes());
        let hash_val = h.finalize();

        let hash_list: Result<Vec<[u8; 32]>, ThanatosError> = Ok(vec![hash_val]);
        assert!(run_check(hash_list, super::system::domain));
    }

    #[test]
    fn system_username_test() {
        let username = super::system::username().expect("Failed to get username");
        let mut h = Sha256::new();
        h.update(username.as_bytes());
        let hash_val = h.finalize();

        let hash_list: Result<Vec<[u8; 32]>, ThanatosError> = Ok(vec![hash_val]);
        assert!(run_check(hash_list, super::system::username));
    }
}
