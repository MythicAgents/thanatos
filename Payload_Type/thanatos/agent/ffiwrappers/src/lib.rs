//! Wrappers around FFI functions

pub mod errors;
pub mod socket;

#[cfg(target_os = "linux")]
pub mod linux;

#[cfg(target_os = "windows")]
pub mod windows;

mod internal {
    pub trait SealedTrait {}
}
