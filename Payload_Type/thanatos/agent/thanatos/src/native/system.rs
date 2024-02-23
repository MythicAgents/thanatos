#[cfg(target_os = "windows")]
pub use crate::os::windows::{
    build_number, domain, hostname, integrity_level, process_name, product, username,
};

#[cfg(target_os = "linux")]
pub use crate::os::linux::{
    domain, hostname, integrity_level, os_release, platform, process_name, username,
};

use crate::proto::checkin::Architecture;
