use std::path::PathBuf;

const PROTO_SRCS: &[&str] = &[
    "config/config.proto",
    "msg/checkin.proto",
    "msg/tasking.proto",
    "msg/mythic.proto",
    "commands/exit.proto",
    "commands/sleep.proto",
];

fn main() {
    let out_dir = std::env::var("OUT_DIR").unwrap();

    let proto_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .canonicalize()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("protobuf");

    let proto_srcs: Vec<PathBuf> = PROTO_SRCS.iter().map(|s| proto_path.join(s)).collect();
    for proto in proto_srcs.iter() {
        println!(
            "cargo::rerun-if-changed={}",
            proto_path.join(proto).to_string_lossy()
        );
    }

    prost_build::Config::new()
        .out_dir(out_dir)
        .include_file("_includes.rs")
        .bytes(&["."])
        .compile_protos(&proto_srcs, &[proto_path])
        .expect("Could not compile protobuf files");
}
