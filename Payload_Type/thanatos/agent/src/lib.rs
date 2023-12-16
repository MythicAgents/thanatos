#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

pub fn entrypoint() {
    let c = config::raw();
    utils::hexdump(c);
}
