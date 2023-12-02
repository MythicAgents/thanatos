//! This module is only imported when targeting Linux hosts
use chrono::{DateTime, Local, NaiveDateTime, Utc};
use std::error::Error;
use std::fs;
use std::io::prelude::*;
use std::os::linux::fs::MetadataExt;
use std::path::Path;
use std::result::Result;
use std::str::FromStr;

use crate::utils::linux::get_user_from_uid;

/// Gets all of the process IDs from `/proc`
pub fn get_process_ids() -> Result<Vec<u32>, Box<dyn Error>> {
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
        if let Some(ending) = path.components().last() {
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
pub fn get_architecture(pid: u32) -> Option<String> {
    // Create the path to the executable symlink
    let path = format!("/proc/{}/exe", pid);

    // Try to read the executable symlink
    let path = fs::read_link(path).ok()?;

    // Open the executable registered with the pid
    let mut f = std::fs::File::open(&path).ok()?;

    // Grab the executable header
    let mut exe_header: [u8; 5] = [0; 5];
    let len = f.read(&mut exe_header).ok()?;
    if len != 5 {
        return None;
    }

    // Check the 5th entry of the executable header to determine if the process is 32 bit or 64 bit
    match exe_header[4] {
        1 => Some("x86".to_string()),
        2 => Some("x64".to_string()),
        _ => None,
    }
}

/// Gets the process name from a pid
/// * `pid` - Pid to get the process name from
pub fn get_proc_name(pid: u32) -> Option<String> {
    // Grab the comm entry for the pid
    let path = format!("/proc/{}/comm", pid);
    let path = Path::new(&path);

    let name = fs::read_to_string(&path).ok()?;
    Some(name.trim_end_matches('\n').to_string())
}

/// Get the user assocated with a process
/// * `pid` - Pid to get the user from
pub fn get_proc_user(pid: u32) -> Option<String> {
    let path = format!("/proc/{}", pid);
    let path = Path::new(&path);

    // Get the uid of the owner for the pid proc entry
    let uid = path.metadata().ok()?.st_uid();

    // Convert the integer uid to a String username
    get_user_from_uid(uid)
}

/// Gets the path to the binary for a process
/// * `pid` - Pid to get the path from
pub fn get_bin_path(pid: u32) -> Option<String> {
    // Try to read the executable symlink
    let path = format!("/proc/{}/exe", pid);
    let path = fs::read_link(path).ok()?;

    // Return where the secutable symlink links to
    Some(path.to_str()?.to_string())
}

/// Get the parent process ID from a process
/// * `pid` - Pid to get the ppid from
pub fn get_ppid(pid: u32) -> Option<u32> {
    // Open the stat information of the process
    let path = format!("/proc/{}/stat", pid);

    // Read the stat information of the process
    let stat = fs::read_to_string(path).ok()?;

    // Split the stat entry of the process to find the ppid
    // More information about this can be found in `man 5 proc`
    let stat: Vec<&str> = stat.split(' ').collect();
    if stat.len() < 52 {
        return None;
    }

    // Because the process name comes before the ppid and can contain spaces, index from the back
    u32::from_str(stat[stat.len() - 48 - 1]).ok()
}

/// Get the command line used to execute the process
/// * `pid` - Pid to get the cmdline from
pub fn get_cmdline(pid: u32) -> Option<String> {
    let path = format!("/proc/{}/cmdline", pid);

    // Read the cmdline replacing null-bytes with spaces
    fs::read_to_string(path)
        .ok()
        .map(|s| s.replace('\0', " ").trim_end_matches(' ').to_string())
}

/// Get the "integrity level" of a process (3 if a root process, 2 for a user process)
/// * `pid` - Pid to get the integrity level from
pub fn get_integrity_level(pid: u32) -> Option<u32> {
    let path = format!("/proc/{}", pid);
    let path = Path::new(&path);

    let uid = path.metadata().ok()?.st_uid();

    // If the process is owned by root return an "integrity level" of 3
    if uid == 0 {
        Some(3)
    } else {
        Some(2)
    }
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

    // Convert the integer unix boot timestamp into the local time
    let btime = DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp_opt(boot_time, 0)?, Utc);
    let btime: DateTime<Local> = DateTime::from(btime);

    // Divide the starttime by the clock ticks
    let starttime = starttime / 100;

    // Add the boot time to the start time of the process to get the start time
    let starttime = btime.checked_add_signed(chrono::Duration::seconds(starttime))?;

    // Return the start time
    Some(starttime.timestamp_millis())
}
