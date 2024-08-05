use std::path::PathBuf;

fn main() {
    let fallback_config = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join(".config");

    let out_path = format!("{}/config.bin", std::env::var("OUT_DIR").unwrap());

    if let Some(config_path) = option_env!("CONFIG") {
        std::fs::copy(config_path, out_path).unwrap();
        println!("cargo:rerun-if-changed={}", config_path);
    } else {
        let _ = std::fs::copy(&fallback_config, out_path);

        println!(
            "cargo:rerun-if-changed={}",
            fallback_config.to_string_lossy().to_string()
        );
    }
}
