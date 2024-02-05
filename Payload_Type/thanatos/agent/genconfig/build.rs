const SRC_CONFIG_PATH: &'static str = env!("SRC_CONFIG");
const CONFIG_PATH: &'static str = env!("CONFIG");

use sha2::Digest;
use std::{io::Write, str::FromStr};

include!("../config/src/structs.rs");

fn hash_string_list(s: &str) -> Vec<[u8; 32]> {
    s.split(",")
        .skip_while(|value| value.is_empty())
        .map(|value| {
            let mut h = sha2::Sha256::new();
            h.update(value.to_lowercase().as_bytes());
            h.finalize().into()
        })
        .collect()
}

fn main() {
    let config_data =
        std::fs::read_to_string(SRC_CONFIG_PATH).expect("Failed to load source config");

    let mut config_map: HashMap<&str, &str> = config_data
        .lines()
        .map(|line| {
            let mut line_split = line.split("=");
            let key = line_split.next().expect("Failed to get key");
            let value = line_split.next().expect("Failed to get value");

            (key, value)
        })
        .collect();

    let config_parsed = ConfigVars {
        init_option: match config_map["INIT_OPTION"] {
            "thread" => InitOption::Thread,
            "fork" => InitOption::Fork,
            "none" | &_ => InitOption::None,
        },

        connection_retries: config_map["CONNECTION_RETRIES"].parse().unwrap(),
        uuid: Uuid::from_str(config_map["UUID"]).unwrap(),
        working_hours_end: config_map["WORKING_HOURS_END"].parse().unwrap(),
        working_hours_start: config_map["WORKING_HOURS_START"].parse().unwrap(),
        spawn_to: config_map.remove("SPAWN_TO").unwrap_or_default(),
        tlsuntrusted: config_map
            .remove("TLS_UNTRUSTED")
            .map(|v| v.parse().ok())
            .flatten()
            .unwrap_or(false),
        domains: hash_string_list(config_map.remove("DOMAIN_LIST").unwrap_or_default()),
        hostnames: hash_string_list(config_map.remove("HOSETNAME_LIST").unwrap_or_default()),
        usernames: hash_string_list(config_map.remove("USERNAME_LIST").unwrap_or_default()),
        http_profile: None,
    };

    let serialized_config = rmp_serde::to_vec(&config_parsed).expect("Failed to serialize config");

    let mut f = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .open(CONFIG_PATH)
        .expect("Failed to open destination config");

    f.write_all(&serialized_config)
        .expect("Failed to write out config");
}
