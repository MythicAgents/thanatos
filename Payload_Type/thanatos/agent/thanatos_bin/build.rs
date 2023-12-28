//! Build script for transforming the configuration

use std::io::Write;

#[cfg(not(feature = "config-builder"))]
fn get_config() -> Vec<u8> {
    let config_path =
        std::env::var("CONFIG_BIN_PATH").expect("Failed to get the 'CONFIG_BIN_PATH' value");

    std::fs::read(config_path).expect("Failed to read the config")
}

fn main() {
    let out_dir = std::env::var("OUT_DIR").expect("Failed to get the 'OUT_DIR' value");
    println!("cargo:rerun-if-changed=../.config");

    let target = std::env::var("TARGET").expect("Failed to get the 'TARGET' value");
    let profile = std::env::var("PROFILE").expect("Failed to get the 'PROFILE' value");
    println!("cargo:rustc-link-search=target/{target}/{profile}");

    println!("cargo:rustc-link-lib=c");
    println!("cargo:rustc-link-lib=gcc_s");

    #[cfg(feature = "config-builder")]
    let config_data = config_builder::load();

    #[cfg(not(feature = "config-builder"))]
    let config_data = get_config();

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
