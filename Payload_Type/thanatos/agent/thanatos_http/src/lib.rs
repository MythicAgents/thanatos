#![forbid(unsafe_code)]
#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use config::ConfigVars;

pub fn entrypoint(config_bytes: &'static [u8]) {
    let config = rmp_serde::from_slice(config_bytes).unwrap();
    thanatos_core::initialize_agent(run_agent, config);
}

fn run_agent(config: ConfigVars) {
    thanatos_core::debug!(config);
}
