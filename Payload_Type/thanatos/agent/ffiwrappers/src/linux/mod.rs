mod hostname;
mod username;

pub use hostname::hostname;
pub use username::username;

/// Returns the libc `errno` value
pub fn libc_errno() -> i32 {
    // SAFETY: `__errno_location` is a pointer to libc's errno value. This pointer
    // is guaranteed to be aligned and non-NULL
    let ec = unsafe { *libc::__errno_location() };

    ec
}
