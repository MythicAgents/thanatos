//! Build script for transforming the configuration

use std::io::Write;

fn main() {
    let out_dir = std::env::var("OUT_DIR").expect("Failed to get the 'OUT_DIR' value");
    println!("cargo:rerun-if-changed=../.config");

    let config_data = config_builder::load();

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
