pub mod errors;

#[cfg(windows)]
mod winhttp;

#[cfg(windows)]
pub use winhttp::profile::HttpProfile;

use serde::Deserialize;

#[derive(Deserialize)]
struct ConfigProxyInfo {
    host: String,
    port: u16,
    user: Option<String>,
    password: Option<String>,
}
