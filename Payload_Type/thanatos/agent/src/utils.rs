use std::path::{Component, PathBuf};

pub mod macros;
pub mod os;
pub mod platform;

/// Convert a windows verbatim path into a nonverbatim one
/// * `path` - Path to clean up
pub fn unverbatim(path: PathBuf) -> PathBuf {
    match path.components().next() {
        Some(Component::Prefix(p)) => {
            if p.kind().is_verbatim() {
                PathBuf::from(
                    path.as_os_str()
                        .to_string_lossy()
                        .trim_start_matches(r"\\?\"),
                )
            } else {
                path.to_path_buf()
            }
        }
        _ => path.to_path_buf(),
    }
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
