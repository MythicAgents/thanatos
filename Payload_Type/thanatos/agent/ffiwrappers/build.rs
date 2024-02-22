use std::path::{Path, PathBuf};

use bindgen::Bindings;

fn generate_bindings(p: impl AsRef<Path>) -> (String, Bindings) {
    let mut a = p.as_ref().ancestors().skip(1);
    let includes = a.next().unwrap().to_string_lossy();
    let includes = includes.trim_start_matches(r"\\?\");

    let bindings = bindgen::Builder::default()
        .clang_arg(format!("-I{}", includes))
        .header(p.as_ref().to_string_lossy())
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .unwrap_or_else(|_| {
            panic!(
                "Failed to generate bindings for {}",
                p.as_ref().to_string_lossy()
            )
        });

    let name = p
        .as_ref()
        .components()
        .last()
        .unwrap()
        .as_os_str()
        .to_string_lossy()
        .to_string();
    let mut name = name.trim_end_matches(".h").to_string();
    name.push_str(".rs");

    (name, bindings)
}

fn main() {
    let target_os =
        std::env::var("CARGO_CFG_TARGET_OS").expect("Failed to get 'CARGO_CFG_TARGET_OS'");

    let header_dir = Path::new("cffiheaders").join(target_os);
    let header_dir = std::fs::read_dir(header_dir).expect("Failed to read header dir");

    for header_path in header_dir.flatten() {
        let header_path = header_path.path().canonicalize().unwrap();

        if header_path.is_file() && header_path.to_string_lossy().ends_with(".h") {
            let (name, bindings) = generate_bindings(header_path);
            let out_path = PathBuf::from(std::env::var("OUT_DIR").unwrap()).join(&name);

            bindings
                .write_to_file(out_path)
                .unwrap_or_else(|_| panic!("Failed to write bindings for {name}"));
        }
    }
}
