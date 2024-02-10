mod ffierrors;

pub use ffierrors::FFIError;

#[derive(Debug)]
pub enum ThanatosError {
    OsError(i32),
    FFIError(FFIError),
    NotDomainJoined,

    #[cfg(target_os = "linux")]
    DbusError(dbus::Error),
}

impl ThanatosError {
    #[cfg(target_os = "windows")]
    pub fn from_windows(e: windows::core::Error) -> Self {
        Self::OsError(e.code().0)
    }
}
