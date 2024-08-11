use std::path::PathBuf;

const PROTO_SRCS: &[&str] = &["config/config.proto", "msg/checkin/checkin.proto"];

fn main() {
    let mut proto_build = prost_build::Config::new();

    let proto_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .canonicalize()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("protobuf");

    let proto_srcs: Vec<PathBuf> = PROTO_SRCS.iter().map(|s| proto_path.join(s)).collect();

    proto_build
        .compile_protos(&proto_srcs, &[proto_path])
        .unwrap();
}
