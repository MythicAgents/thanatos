//! Core Thanatos agent

/// Main entrypoint for the agent
pub fn entrypoint() {
    let c = config::raw();
    utils::hexdump(c);
}
