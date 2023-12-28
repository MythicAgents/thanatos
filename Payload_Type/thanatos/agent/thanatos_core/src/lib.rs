//! Core Thanatos agent

/// Main entrypoint for the agent
#[no_mangle]
pub extern "C" fn entrypoint(config_data: *const u8, config_data_len: usize) {
    let config_data = unsafe { std::slice::from_raw_parts(config_data, config_data_len) };
    let config: config::ConfigVars<'_> = rmp_serde::from_slice(config_data).unwrap();
    dbg!(&config);
}
