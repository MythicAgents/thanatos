//! Core Thanatos agent

use utils::hexdump::{self, HexdumpFormat};

/// Main entrypoint for the agent
pub fn entrypoint(config_data: &[u8]) {
    hexdump::hexdump(config_data, HexdumpFormat::XxdColored);
}
