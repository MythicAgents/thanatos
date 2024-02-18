#[derive(Debug)]
pub enum FfiError {
    OsError(i32),
    NoNullTerminator,
    InteriorNull,

    #[cfg(target_os = "linux")]
    GaiError(EaiError),
    NonNullPointer,
    CanonNameNotFound,

    #[cfg(target_os = "linux")]
    NoGroupMembership,
}

impl FfiError {
    #[cfg(target_os = "linux")]
    pub fn os_error() -> Self {
        Self::OsError(libc_errno())
    }

    #[cfg(target_os = "windows")]
    pub fn from_windows_error(e: windows::core::Error) -> Self {
        Self::OsError(e.code().0)
    }
}

#[derive(Debug)]
#[repr(i32)]
#[cfg(target_os = "linux")]
pub enum EaiError {
    Other(i32),
    System(i32),
    BadFlags = libc::EAI_BADFLAGS,
    NoName = libc::EAI_NONAME,
    Again = libc::EAI_AGAIN,
    Fail = libc::EAI_FAIL,
    Family = libc::EAI_FAMILY,
    SockType = libc::EAI_SOCKTYPE,
    Service = libc::EAI_SERVICE,
    Memory = libc::EAI_MEMORY,
    Overflow = libc::EAI_OVERFLOW,
}

#[cfg(target_os = "linux")]
impl EaiError {
    pub fn from_code(code: i32) -> EaiError {
        match code {
            libc::EAI_BADFLAGS => Self::BadFlags,
            libc::EAI_NONAME => Self::NoName,
            libc::EAI_AGAIN => Self::Again,
            libc::EAI_FAIL => Self::Fail,
            libc::EAI_FAMILY => Self::Family,
            libc::EAI_SOCKTYPE => Self::SockType,
            libc::EAI_SERVICE => Self::Service,
            libc::EAI_MEMORY => Self::Memory,
            libc::EAI_OVERFLOW => Self::Overflow,
            libc::EAI_SYSTEM => Self::System(libc_errno()),
            _ => Self::Other(code),
        }
    }
}

/// Returns the libc `errno` value
#[cfg(target_os = "linux")]
fn libc_errno() -> i32 {
    // SAFETY: `__errno_location` is a pointer to libc's errno value. This pointer
    // is guaranteed to be aligned and non-NULL
    unsafe { *libc::__errno_location() }
}
