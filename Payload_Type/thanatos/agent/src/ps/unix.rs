//! This module is only imported when targeting Linux hosts
use std::{fs, io::prelude::*, os::unix::fs::MetadataExt, path::Path, str::FromStr};

use chrono::DateTime;

use crate::{
    agent::AgentTask,
    utils::{
        os::unix::{IntegrityLevelExt, PasswdEntry},
        platform::{self, IntegrityLevel},
    },
};

/// Gets all of the process IDs from `/proc`
fn get_process_ids() -> Result<Vec<u32>, std::io::Error> {
    // Create the vec of pids
    let mut pids: Vec<u32> = Vec::new();

    // Iterate over each entry in `/proc`
    for entry in std::fs::read_dir("/proc")? {
        // If there was an error reading the entry, skip over it
        let entry = if let Ok(entry) = entry {
            entry
        } else {
            continue;
        };

        let path = entry.path();

        // Check if the `/proc` entry is a directory
        if !path.is_dir() {
            continue;
        }

        // Try to grab the last element of the path
        if let Some(ending) = path.components().next_back() {
            // Convert the last element of the path to a string
            if let Some(ending) = ending.as_os_str().to_str() {
                // Try to parse the path into an integer and add it to the list of pids
                if let Ok(pid) = u32::from_str(ending) {
                    pids.push(pid);
                }
            }
        }
    }

    // Return the process ids
    Ok(pids)
}

/// Gets the architecture of the process
/// * `pid` - Pid to get the architecture from
fn pid_architecture(pid: u32) -> Result<Option<String>, std::io::Error> {
    // Create the path to the executable symlink
    let path = format!("/proc/{}/exe", pid);

    // Try to read the executable symlink
    let path = fs::read_link(path)?;

    // Open the executable registered with the pid
    let mut f = std::fs::File::open(path)?;

    // Grab the executable header
    let mut exe_header: [u8; 5] = [0; 5];
    let len = f.read(&mut exe_header)?;
    if len != 5 {
        return Ok(None);
    }

    // Check the 5th entry of the executable header to determine if the process is 32 bit or 64 bit
    match exe_header[4] {
        1 => Ok(Some("x86".to_string())),
        2 => Ok(Some("x64".to_string())),
        _ => Ok(None),
    }
}

/// Gets the process name from a pid
/// * `pid` - Pid to get the process name from
fn get_proc_name(pid: u32) -> Result<String, std::io::Error> {
    // Grab the comm entry for the pid
    let path = format!("/proc/{}/comm", pid);
    let path = Path::new(&path);

    let name = fs::read_to_string(path)?;
    Ok(name.trim_end_matches('\n').to_string())
}

/// Get the user assocated with a process
/// * `pid` - Pid to get the user from
fn get_proc_user(pid: u32) -> Result<String, std::io::Error> {
    let path = format!("/proc/{}", pid);
    let path = Path::new(&path);

    // Get the uid of the owner for the pid proc entry
    let uid = path.metadata()?.uid();

    // Convert the integer uid to a String username
    PasswdEntry::getpwuid(uid)
        .and_then(|entry| entry.ok_or_else(|| std::io::Error::from_raw_os_error(libc::ENOENT)))
        .map(|entry| entry.name().to_string_lossy().to_string())
}

/// Gets the path to the binary for a process
/// * `pid` - Pid to get the path from
fn get_bin_path(pid: u32) -> Result<String, std::io::Error> {
    // Try to read the executable symlink
    let path = format!("/proc/{}/exe", pid);
    let path = fs::read_link(path)?;

    // Return where the executable symlink links to
    Ok(path.to_string_lossy().to_string())
}

/// Get the parent process ID from a process
/// * `pid` - Pid to get the ppid from
fn get_ppid(pid: u32) -> Result<u32, std::io::Error> {
    // Open the stat information of the process
    let path = format!("/proc/{}/stat", pid);

    // Read the stat information of the process
    let stat = fs::read_to_string(path)?;

    // Split the stat entry of the process to find the ppid
    // More information about this can be found in `man 5 proc`
    let stat: Vec<&str> = stat.split(' ').collect();
    if stat.len() < 52 {
        return Err(std::io::Error::from_raw_os_error(libc::ENOENT));
    }

    // Because the process name comes before the ppid and can contain spaces, index from the back
    u32::from_str(stat[stat.len() - 48 - 1]).map_err(std::io::Error::other)
}

/// Get the command line used to execute the process
/// * `pid` - Pid to get the cmdline from
fn get_cmdline(pid: u32) -> Result<String, std::io::Error> {
    let path = format!("/proc/{}/cmdline", pid);

    // Read the cmdline replacing null-bytes with spaces
    fs::read_to_string(path).map(|s| s.replace('\0', " ").trim_end_matches(' ').to_string())
}

/// Get the "integrity level" of a process (3 if a root process, 2 for a user process)
/// * `pid` - Pid to get the integrity level from
fn get_integrity_level(pid: u32) -> Result<IntegrityLevel, std::io::Error> {
    let path = format!("/proc/{}", pid);
    let path = Path::new(&path);

    let uid = path.metadata()?.uid();
    Ok(IntegrityLevel::from_uid(uid))
}

/// Get the start time of a process
/// * `pid` - Pid to get the start time from
/// * `boot_time` - Boot time of the machine
pub fn get_start_time(pid: u32, boot_time: i64) -> Option<i64> {
    if boot_time == 0 {
        return None;
    }

    // Read the stat information from the proc entry
    let proc_stat = fs::read_to_string(format!("/proc/{}/stat", pid)).ok()?;
    let proc_stat: Vec<&str> = proc_stat.split(' ').collect();

    if proc_stat.len() < 52 {
        return None;
    }

    // Grab the value at the starttime index
    let starttime = i64::from_str(proc_stat[proc_stat.len() - 30 - 1]).ok()?;

    // Divide the starttime by the clock ticks
    let starttime = starttime / 100;

    // Get the boot time
    let boot_timestamp = DateTime::from_timestamp(boot_time, 0)?;

    // Add the boot time to the start time of the process to get the start time
    let starttime = boot_timestamp.checked_add_signed(chrono::Duration::seconds(starttime))?;

    // Return the start time
    Some(starttime.timestamp_millis())
}

/// Grab the process listing for linux hosts
/// * `task` - Task used to invoke the process listing
pub fn get_process_list(task: &AgentTask) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    // Create a vec for each process
    let mut listing = Vec::<super::ProcessListingEntry>::new();

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
        listing.push(super::ProcessListingEntry {
            process_id: pid,
            architecture: pid_architecture(pid).ok().flatten().unwrap_or_default(),
            name: get_proc_name(pid).ok(),
            user: get_proc_user(pid).ok(),
            bin_path: get_bin_path(pid).ok(),
            parent_process_id: get_ppid(pid).ok(),
            command_line: get_cmdline(pid).ok(),
            integrity_level: get_integrity_level(pid).ok(),
            start_time: get_start_time(pid, boot_time),
            description: None,
            signer: None,
        });
    }

    // Create the process listing output with the platform
    let output = super::ProcessListing {
        platform: platform::generic_name(),
        processes: &listing,
    };

    // Return the process listing
    Ok(serde_json::json!({
        "task_id": task.id,
        "status": "success",
        "completed": true,
        "user_output": serde_json::to_string(&output)?,
        "processes": serde_json::to_value(listing)?,
    }))
}
