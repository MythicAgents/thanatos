//! Contains the configuration values for agent

mod structs;

use structs::{ConfigVars, HttpConfigVars};

const CONFIG_BYTES: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/config.bin"));
const HTTP_CONFIG_BYTES: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/config.http.bin"));

fn get_config() -> ConfigVars<'static> {
    rmp_serde::from_slice(CONFIG_BYTES).unwrap()
}

fn get_http_config() -> HttpConfigVars<'static> {
    rmp_serde::from_slice(HTTP_CONFIG_BYTES).unwrap()
}
