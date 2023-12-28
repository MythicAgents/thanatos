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
    Daemonize = 2,
}

/// HTTP profile configuration variables
#[derive(Serialize, Deserialize, Debug)]
pub struct HttpConfigVars<'a> {
    callback_host: &'a str,
    callback_interval: usize,
    callback_jitter: u16,
    callback_port: u16,
    get_uri: &'a str,
    headers: HashMap<&'a str, &'a str>,
    killdate: usize,
    post_uri: &'a str,
    query_path_name: &'a str,
}

/// Payload configuration variables
#[derive(Serialize, Deserialize, Debug)]
pub struct ConfigVars<'a> {
    uuid: Uuid,
    init_option: InitOption,
    working_hours_start: u64,
    working_hours_end: u64,
    connection_retries: usize,
    domains: Vec<[u8; 32]>,
    hostnames: Vec<[u8; 32]>,
    usernames: Vec<[u8; 32]>,
    tlsselfsigned: bool,
    spawn_to: &'a str,
    profile: HttpConfigVars<'a>,
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
}
