use std::{
    io::{BufWriter, Write},
    path::PathBuf,
};

fn main() {
    if std::env::var_os("CARGO_FEATURE_USER").is_none() {
        return;
    }

    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());

    let entrypoint_name = std::env::var("THANATOS_SHARED_ENTRYPOINT")
        .unwrap_or_else(|_| panic!("THANATOS_SHARED_ENTRYPOINT environment variable is not set."));

    let mut userfile = BufWriter::new(std::fs::File::create(out_dir.join("user.rs")).unwrap());

    writeln!(
        userfile,
        r#"
#[unsafe(no_mangle)]
extern "system" fn {entrypoint}() {{
    let _ = thanatos::real_main();
}}
"#,
        entrypoint = entrypoint_name,
    )
    .unwrap();
}
