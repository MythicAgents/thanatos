use crate::agent::AgentTask;
use serde::Serialize;
use serde_json::json;
use std::error::Error;
use std::result::Result;

// Import the necessary modules based on target platform
cfg_if::cfg_if! {
    if #[cfg(target_os = "linux")] {
        use crate::utils::linux::whoami;
        mod linux_ps;
        use linux_ps::*;
    } else if #[cfg(target_os = "windows")] {
        use crate::utils::windows::whoami;
        mod windows_ps;
        use windows_ps::*;
    }
}

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
    pub integrity_level: Option<u32>,

    /// Start time of the process
    pub start_time: Option<i64>,

    /// Description of the process
    pub description: Option<String>,

    /// Signer of the process binary
    pub signer: Option<String>,
}

/// Grab the process listing for windows hosts
/// * `task` - Task used to invoke the process listing
#[cfg(target_os = "windows")]
pub fn get_process_list(task: &AgentTask) -> Result<serde_json::Value, Box<dyn Error>> {
    // Get the process information
    let listing = process_info()?;

    // Create the output with the platform
    let output = ProcessListing {
        platform: whoami::generic_platform(),
        processes: &listing,
    };

    // Return the process listing
    Ok(json!({
        "task_id": task.id,
        "status": "success",
        "completed": true,
        "user_output": serde_json::to_string(&output)?,
        "processes": serde_json::to_value(listing)?,
    }))
}

/// Grab the process listing for linux hosts
/// * `task` - Task used to invoke the process listing
#[cfg(target_os = "linux")]
pub fn get_process_list(task: &AgentTask) -> Result<serde_json::Value, Box<dyn Error>> {
    // Create a vec for each process
    let mut listing: Vec<ProcessListingEntry> = Vec::new();

    // Get the boot time of the machine. Used for getting the process start time
    let boot_time: i64 = if let Ok(s) = std::fs::read_to_string("/proc/stat") {
        let start = s.find("btime ").unwrap();
        let end = s.find("\nprocesses ").unwrap();

        s.get(start..end)
            .unwrap_or("btime 0")
            .split(' ')
            .next_back()
            .unwrap_or("0")
            .parse()
            .unwrap_or(0)
    } else {
        0
    };

    // Iterate over each process id and add the necessary information
    for pid in get_process_ids()? {
        listing.push(ProcessListingEntry {
            process_id: pid,
            architecture: get_architecture(pid).unwrap_or_default(),
            name: get_proc_name(pid),
            user: get_proc_user(pid),
            bin_path: get_bin_path(pid),
            parent_process_id: get_ppid(pid),
            command_line: get_cmdline(pid),
            integrity_level: get_integrity_level(pid),
            start_time: get_start_time(pid, boot_time),
            description: None,
            signer: None,
        });
    }

    // Create the process listing output with the platform
    let output = ProcessListing {
        platform: whoami::generic_platform(),
        processes: &listing,
    };

    // Return the process listing
    Ok(json!({
        "task_id": task.id,
        "status": "success",
        "completed": true,
        "user_output": serde_json::to_string(&output)?,
        "processes": serde_json::to_value(listing)?,
    }))
}
