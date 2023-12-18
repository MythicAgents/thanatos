//! Core Thanatos agent

/// Main entrypoint for the agent
pub fn entrypoint(config_data: &[u8]) {
    let config: config::ConfigVars<'_> = rmp_serde::from_slice(config_data).unwrap();
    dbg!(&config);
}
