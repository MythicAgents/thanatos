//! Wrappers around FFI functions

pub mod errors;

#[cfg(target_os = "linux")]
pub mod linux;

#[cfg(target_os = "windows")]
pub mod windows;

mod internal {
    // This is used but clippy complains
    // TODO: Need to rewrite all of this
    #[allow(dead_code)]
    pub trait SealedTrait {}
}
