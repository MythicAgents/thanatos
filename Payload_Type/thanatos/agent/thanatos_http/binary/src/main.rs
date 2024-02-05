#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

const CONFIG: &[u8] = include_bytes!(env!("CONFIG"));

fn main() {
    thanatos_http::entrypoint(CONFIG);
}
