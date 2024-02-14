use pb_rs::{types::FileDescriptor, ConfigBuilder};
use std::path::{Path, PathBuf};

fn main() {
    let config_proto = Path::new(&std::env::var("CARGO_MANIFEST_DIR").unwrap())
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("mythic")
        .join("protos")
        .join("config.proto");

    let out_proto = Path::new(&std::env::var("CARGO_MANIFEST_DIR").unwrap())
        .join("src")
        .join("proto");

    if !config_proto.exists() && !out_proto.exists() {
        panic!("Failed to generate protos");
    }

    if config_proto.exists() {
        println!("cargo:rerun-if-changed={}", config_proto.to_str().unwrap());
    }

    if out_proto.exists() {
        std::fs::remove_dir_all(&out_proto).unwrap();
    }

    std::fs::DirBuilder::new().create(&out_proto).unwrap();

    let builder = ConfigBuilder::new(&[config_proto], Some(&out_proto), None, &[])
        .unwrap()
        .single_module(true);
    FileDescriptor::run(&builder.build()).unwrap();

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
