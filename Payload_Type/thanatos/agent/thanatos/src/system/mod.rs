pub mod time;

#[cfg(target_os = "linux")]
pub use crate::os::linux::{domain, hostname, username};

#[cfg(target_os = "windows")]
pub use crate::os::windows::{domain, hostname, username};
