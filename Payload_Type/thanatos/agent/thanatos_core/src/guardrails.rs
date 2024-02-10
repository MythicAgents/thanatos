#[cfg(target_os = "linux")]
use crate::native::linux::system;

#[cfg(target_os = "windows")]
use crate::native::windows::system;

cfg_if::cfg_if! {
    if #[cfg(feature = "crypto-system")] {
        use cryptolib::hash::system::Sha256;
    } else {
        use cryptolib::hash::internal::Sha256;
    }
}

#[cfg(target_os = "linux")]
pub fn check_domain(domains: &[[u8; 32]]) -> bool {
    let check_domains = match system::domains() {
        Ok(check_domains) => check_domains,
        Err(_) => return false,
    };

    check_domains
        .into_iter()
        .any(|check_domain| check_hashlist_with(domains, &check_domain))
}

#[cfg(target_os = "windows")]
pub fn check_domain(domains: &[[u8; 32]]) -> bool {
    let domain = match system::domain() {
        Ok(domain) => domain,
        Err(_) => return false,
    };

    check_hashlist_with(domains, &domain)
}

pub fn check_hostname(hostnames: &[[u8; 32]]) -> bool {
    let hostname = match system::hostname() {
        Ok(hostname) => hostname,
        Err(_) => return false,
    };

    check_hashlist_with(hostnames, &hostname)
}

pub fn check_username(usernames: &[[u8; 32]]) -> bool {
    let username = match system::username() {
        Ok(username) => username,
        Err(_) => return false,
    };

    check_hashlist_with(usernames, &username)
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
