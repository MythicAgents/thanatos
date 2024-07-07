#![forbid(unsafe_code)]

use prost::Message;

mod logging;
mod native;
mod os;

pub fn entrypoint(config: &[u8]) {
    let _agent_config = thanatos_protos::config::Config::decode(config).unwrap();
}
