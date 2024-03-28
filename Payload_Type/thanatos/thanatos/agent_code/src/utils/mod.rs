use std::{
    ffi::OsStr,
    path::{Component, Path, PathBuf},
};

#[cfg(target_os = "linux")]
pub mod linux;

#[cfg(target_os = "windows")]
pub mod windows;

// Used for getting the local internal IP address
// This just binds to a socket and checks what the interface IP address is not the best
// way to do it but there is no more intuitive "better" way
pub mod local_ipaddress {
    use std::net::UdpSocket;

    /// Get the local IP address of the machine
    pub fn get() -> Option<String> {
        let socket = UdpSocket::bind("0.0.0.0:0").ok()?;
        socket.connect("8.8.8.8:80").ok()?;
        Some(socket.local_addr().ok()?.ip().to_string())
    }
}

/// Convert a windows verbatim path into a nonverbatim one
/// * `path` - Path to clean up
pub fn unverbatim(path: impl AsRef<Path>) -> PathBuf {
    match path.as_ref().components().next() {
        Some(Component::Prefix(p)) => {
            if p.kind().is_verbatim() {
                PathBuf::from(
                    path.as_ref()
                        .as_os_str()
                        .to_string_lossy()
                        .trim_start_matches(r"\\?\"),
                )
            } else {
                path.as_ref().to_path_buf()
            }
        }
        _ => path.as_ref().to_path_buf(),
    }
}

pub fn cleanpath(path: impl AsRef<Path>) -> PathBuf {
    let mut cleaned: Vec<&OsStr> = Vec::new();

    let mut components = path.as_ref().components().rev();
    while let Some(p) = components.next() {
        match p {
            Component::ParentDir => {
                if components.next().is_none() {
                    break;
                }
            }
            v => cleaned.push(v.as_os_str()),
        }
    }

    PathBuf::from_iter(cleaned.iter().rev())
}

/// Helper function to convert a linux integer mode into a human-readable format
/// * `mode` - Integer posix mode
pub fn parse_linux_mode(mode: u32) -> String {
    let mut str_mode = String::new();

    let perms = [(mode & 0o700) >> 6, (mode & 0o70) >> 3, mode & 7];

    // Check if it is a directory
    if mode & 0o40000 != 0 {
        str_mode.push('d');
    } else {
        str_mode.push('-');
    }

    for (i, p) in perms.iter().enumerate() {
        // Check if the read bit is set
        if (p & (1 << 2)) != 0 {
            str_mode.push('r');
        } else {
            str_mode.push('-');
        }

        // Check if the write bit is set
        if (p & (1 << 1)) != 0 {
            str_mode.push('w');
        } else {
            str_mode.push('-');
        }

        match i {
            // Check the owner sticky bit or executable bit
            0 => {
                if (mode & 0o4000) != 0 {
                    str_mode.push('S');
                } else if (p & 1) != 0 {
                    str_mode.push('x');
                } else {
                    str_mode.push('-');
                }
            }

            // Check the group sticky bit or executable bit
            1 => {
                if (mode & 0o2000) != 0 {
                    str_mode.push('S');
                } else if (p & 1) != 0 {
                    str_mode.push('x');
                } else {
                    str_mode.push('-');
                }
            }

            // Check the other sticky bit or executable bit
            2 => {
                if (mode & 0o1000) != 0 {
                    str_mode.push('T');
                } else if (p & 1) != 0 {
                    str_mode.push('x');
                } else {
                    str_mode.push('-');
                }
            }
            _ => {}
        }
    }

    str_mode
}

/// Macro used for formatting a string into the Mythic error json format
/// * `task_id` - Id of the task
/// * `error` - Error to display in the Mythic task
#[macro_export]
macro_rules! mythic_error {
    ($tid:expr, $errmsg:expr) => {
        serde_json::json!({
            "task_id": $tid,
            "status": "error",
            "user_output": $errmsg,
            "completed": true,
        })
    };
}

/// Macro used for formatting a string into the Mythic success json format
/// * `task_id` - Id of the task
/// * `output` - Message for the task output
#[macro_export]
macro_rules! mythic_success {
    ($tid:expr, $out:expr) => {
        serde_json::json!({
            "task_id": $tid,
            "status": "success",
            "user_output": $out,
            "completed": true,
        })
    };
}

/// Macro used for formatting a string into the Mythic json format with a custom status message
/// * `task_id` - Id of the task
/// * `status` - Status message for the task
/// * `output` - Message for the task output
#[macro_export]
macro_rules! mythic_continued {
    ($tid:expr, $stat:expr, $out:expr) => {
        serde_json::json!({
            "task_id": $tid,
            "status": $stat,
            "user_output": $out,
            "completed": false,
        })
    };
}
