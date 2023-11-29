/// Get the current working directory using the rust std library
#[cfg(not(feature = "load"))]
pub fn pwd() -> Result<String, std::io::Error> {
    std::env::current_dir().map(|p| p.to_string_lossy().to_string())
}
