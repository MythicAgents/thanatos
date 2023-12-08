use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize)]
pub(crate) struct HttpConfigVars<'a> {
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

#[derive(Serialize, Deserialize)]
pub(crate) struct ConfigVars<'a> {
    connection_retries: usize,
    uuid: &'a str,
    working_hours_end: usize,
    working_hours_start: usize,
}
