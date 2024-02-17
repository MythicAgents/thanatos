#![allow(unused)]

#[cfg(target_os = "linux")]
use crate::native::linux::system;

use crate::native::linux::system::domain;
#[cfg(target_os = "windows")]
use crate::native::windows::system;

#[cfg(feature = "crypto-system")]
use cryptolib::hash::system::Sha256;

#[cfg(not(feature = "crypto-system"))]
use cryptolib::hash::internal::Sha256;

use config::ConfigVars;

#[inline(always)]
pub fn run_guardrails(agent_config: &ConfigVars) -> bool {
    #[cfg(feature = "usernamecheck")]
    if !agent_config
        .usernames()
        .ok()
        .and_then(|usernames_list| {
            let current_user = system::username().ok()?;
            Some(check_hashlist_with(&usernames_list, &current_user))
        })
        .is_some_and(|r| r)
    {
        return false;
    }

    #[cfg(feature = "hostnamecheck")]
    if !agent_config
        .hostnames()
        .ok()
        .and_then(|hostnames_list| {
            let current_hostname = system::hostname().ok()?;
            Some(check_hashlist_with(&hostnames_list, &current_hostname))
        })
        .is_some_and(|r| r)
    {
        return false;
    }

    #[cfg(feature = "domaincheck")]
    if !agent_config
        .domains()
        .ok()
        .and_then(|domains_list| {
            let current_domain = system::domain().ok()?;
            Some(check_hashlist_with(&domains_list, &current_domain))
        })
        .is_some_and(|r| r)
    {
        return false;
    }

    true
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
    use super::check_hashlist_with;

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
}
