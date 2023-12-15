//! Contains the configuration values for agent

use structs::ConfigVars;
use utils::uuid::Uuid;

pub use structs::InitOption;

mod structs;

#[link_section = ".rsrc"]
static CONFIG_BYTES: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/config.bin"));

macro_rules! get_field {
    ($field:ident) => {
        rmp_serde::from_slice::<ConfigVars<'_>>(CONFIG_BYTES)
            .unwrap()
            .$field
    };
}

/// Get the raw confuguration data
pub fn raw() -> &'static [u8] {
    CONFIG_BYTES
}

/// Returns the configured payload uuid
pub fn uuid() -> Uuid {
    get_field!(uuid)
}

/// Returns the configured init option
pub fn init_option() -> InitOption {
    get_field!(init_option)
}

/// Returns the working hours start time
pub fn working_hours_start() -> std::time::Duration {
    std::time::Duration::from_secs(get_field!(working_hours_start))
}

/// Returns the working hours end time
pub fn working_hours_end() -> std::time::Duration {
    std::time::Duration::from_secs(get_field!(working_hours_end))
}

/// Returns the connection retries
pub fn connection_retries() -> usize {
    get_field!(connection_retries)
}

/// Returns the list of domains to check
pub fn domains() -> Vec<[u8; 32]> {
    get_field!(domains)
}

/// Returns the list of hostnames to check
pub fn hostnames() -> Vec<[u8; 32]> {
    get_field!(hostnames)
}

/// Returns the list of usernames to check
pub fn usernames() -> Vec<[u8; 32]> {
    get_field!(usernames)
}

/// Returns the option of whether the agent should connect to self signed TLS certificates
pub fn tlsselfsigned() -> bool {
    get_field!(tlsselfsigned)
}

/// Returns the configured spawn to value
pub fn spawn_to() -> String {
    get_field!(spawn_to).to_string()
}
