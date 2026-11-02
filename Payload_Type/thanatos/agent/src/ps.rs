use serde::Serialize;

#[cfg_attr(unix, path = "ps/unix.rs")]
#[cfg_attr(windows, path = "ps/windows.rs")]
mod imp;

pub use imp::get_process_list;

use crate::utils::platform::IntegrityLevel;

#[derive(Default, Serialize)]
pub struct ProcessListing<'a> {
    platform: String,
    processes: &'a [ProcessListingEntry],
}

/// Struct holding the information for a process listing
#[derive(Default, Serialize)]
pub struct ProcessListingEntry {
    /// Process ID of the process
    pub process_id: u32,

    /// Architecture of the process (x86, x64)
    pub architecture: String,

    /// Name of the process
    pub name: Option<String>,

    /// User associated with the process
    pub user: Option<String>,

    /// Path to the binary associated with the process
    pub bin_path: Option<String>,

    /// Parent process ID
    pub parent_process_id: Option<u32>,

    /// Command line used to invoke the process
    pub command_line: Option<String>,

    /// Integrity level of the process
    pub integrity_level: Option<IntegrityLevel>,

    /// Start time of the process
    pub start_time: Option<i64>,

    /// Description of the process
    pub description: Option<String>,

    /// Signer of the process binary
    pub signer: Option<String>,
}
