#[cfg_attr(unix, path = "getprivs/unix.rs")]
#[cfg_attr(windows, path = "getprivs/windows.rs")]
mod imp;

pub use imp::get_privileges;
