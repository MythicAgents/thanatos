use std::path::{Path, PathBuf};

fn main() {
    let proto_dir = Path::new(&std::env::var("CARGO_MANIFEST_DIR").unwrap())
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("mythic")
        .join("protos")
        .join("config");

    let config_proto = proto_dir.join("config.proto");

    if config_proto.exists() {
        println!("cargo:rerun-if-changed={}", config_proto.to_str().unwrap());
    }

    let out_dir = Path::new("src/");
    if out_dir.exists() {
        let _ = std::fs::remove_dir(out_dir);
    }

    let mut proto_build = prost_build::Config::new();
    proto_build.out_dir(out_dir);

    proto_build
        .compile_protos(&[config_proto], &[proto_dir])
        .expect("Failed to compile config.proto");

    if std::env::var("CARGO_FEATURE_FULL").is_ok() {
        let config_bin = match std::env::var("CONFIG") {
            Ok(c) => Some(PathBuf::from(c)),
            Err(_) => {
                let c = Path::new(&std::env::var("CARGO_MANIFEST_DIR").unwrap())
                    .parent()
                    .unwrap()
                    .join(".config.bin");
                if c.exists() {
                    Some(c.canonicalize().unwrap())
                } else {
                    None
                }
            }
        }
        .expect("Failed to find 'CONFIG' or '.config.bin'");

        let out_config_path = Path::new(&std::env::var("OUT_DIR").unwrap()).join("config.bin");
        std::fs::copy(&config_bin, out_config_path).expect("Failed to copy config.bin");
        println!("cargo:rerun-if-changed={}", config_bin.to_str().unwrap());
    }
}
