use ffiwrappers::errors::FfiError;

#[derive(Debug)]
pub enum ThanatosError {
    OsError(i32),
    FFIError(FfiError),
    NotDomainJoined,
    PassedKilldate,
    GuardrailChecksFailed,

    IoError(std::io::Error),

    ConfigParseError,
    OutOfProfiles,
}

impl ThanatosError {
    #[cfg(target_os = "windows")]
    pub fn from_windows(e: windows::core::Error) -> Self {
        Self::OsError(e.code().0)
    }
}
