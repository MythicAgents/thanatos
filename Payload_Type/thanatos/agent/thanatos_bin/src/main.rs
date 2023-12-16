//! Binary executable version of the thanatos agent. Wrapper for loading the main thanatos agent

#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

fn main() {
    thanatos_core::entrypoint();
}
