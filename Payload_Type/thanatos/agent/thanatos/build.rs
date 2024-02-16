use std::path::Path;

use pb_rs::{types::FileDescriptor, ConfigBuilder};

fn main() {
    let checkin_proto = Path::new(&std::env::var("CARGO_MANIFEST_DIR").unwrap())
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("mythic")
        .join("protos")
        .join("checkin.proto");

    let out_proto = Path::new(&std::env::var("CARGO_MANIFEST_DIR").unwrap())
        .join("src")
        .join("proto");

    if !checkin_proto.exists() && !out_proto.exists() {
        panic!("Failed to generate protos");
    }

    if checkin_proto.exists() {
        println!("cargo:rerun-if-changed={}", checkin_proto.to_str().unwrap());
    }

    if out_proto.exists() {
        std::fs::remove_dir_all(&out_proto).unwrap();
    }

    std::fs::DirBuilder::new().create(&out_proto).unwrap();

    let builder = ConfigBuilder::new(&[checkin_proto], Some(&out_proto), None, &[])
        .unwrap()
        .single_module(true)
        .dont_use_cow(true);
    FileDescriptor::run(&builder.build()).unwrap();
}
