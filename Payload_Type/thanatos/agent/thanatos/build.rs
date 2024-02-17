use std::path::Path;

fn main() {
    let proto_dir = Path::new(&std::env::var("CARGO_MANIFEST_DIR").unwrap())
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("mythic")
        .join("protos");

    let checkin_proto = proto_dir.join("msg").join("checkin.proto");

    if checkin_proto.exists() {
        println!(
            "cargo:rerun-if-changed={}",
            checkin_proto
                .canonicalize()
                .expect("Failed to find checkin.proto")
                .to_str()
                .unwrap()
        );
    }

    prost_build::compile_protos(&[checkin_proto], &[proto_dir]).expect("Failed to compile protos");
}
