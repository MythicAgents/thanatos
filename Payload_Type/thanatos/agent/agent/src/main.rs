#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]
#![warn(missing_docs)]
#![warn(clippy::missing_docs_in_private_items)]

//! Main file for the agent

/// Entry point for the built executables
fn main() {
    thanatoslib::thanatos_init();
}
