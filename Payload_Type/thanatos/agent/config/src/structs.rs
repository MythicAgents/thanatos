use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use std::collections::HashMap;
use utils::uuid::Uuid;

/// Configuration option for the initial payload execution
#[derive(Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum InitOption {
    /// Payload should not do anything special when executed
    None,

    /// Payload should run in a new thread
    Thread,

    /// Payload should fork to the background
    Daemonize,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct HttpConfigVars<'a> {
    pub(crate) callback_host: &'a str,
    pub(crate) callback_interval: usize,
    pub(crate) callback_jitter: u16,
    pub(crate) callback_port: u16,
    pub(crate) get_uri: &'a str,
    pub(crate) headers: HashMap<&'a str, &'a str>,
    pub(crate) killdate: usize,
    pub(crate) post_uri: &'a str,
    pub(crate) query_path_name: &'a str,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct ConfigVars<'a> {
    pub(crate) uuid: Uuid,
    pub(crate) init_option: InitOption,
    pub(crate) working_hours_start: u64,
    pub(crate) working_hours_end: u64,
    pub(crate) connection_retries: usize,
    pub(crate) domains: Vec<[u8; 32]>,
    pub(crate) hostnames: Vec<[u8; 32]>,
    pub(crate) usernames: Vec<[u8; 32]>,
    pub(crate) tlsselfsigned: bool,
    pub(crate) spawn_to: &'a str,
    pub(crate) profile: HttpConfigVars<'a>,
}
