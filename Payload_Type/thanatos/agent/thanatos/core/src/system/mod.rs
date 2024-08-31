pub mod os;

#[cfg(target_os = "linux")]
pub use os::linux::*;

#[cfg(target_os = "windows")]
pub use os::windows::*;
