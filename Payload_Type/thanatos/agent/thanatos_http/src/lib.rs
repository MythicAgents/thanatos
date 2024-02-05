#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use config::ConfigVars;

pub fn entrypoint(config_bytes: &[u8]) {
    let agent_config: ConfigVars = rmp_serde::from_slice(config_bytes).unwrap();
    dbg!(agent_config);
}
