use windows::core::HRESULT;

use crate::{agent::errors::AgentError, crypto::errors::CryptoError};

const _: () = assert!(
    std::mem::size_of::<ThanatosError>() >= 8,
    "ThanatosError is larger than 8 bytes"
);

#[derive(Debug)]
pub enum ThanatosError {
    /// SystemTime returned an error
    SystemTimeError,

    Agent(AgentError),

    /// Error doing cryptography routine
    Crypto(CryptoError),

    /// Error deserializing/serializing JSON data
    Json(serde_json::Error),

    /// Failed to parse string
    InvalidString,

    /// Path being parsed is not a file
    PathNotAFile,

    /// Windows error code
    #[cfg(windows)]
    WinError(HRESULT),
}

impl ThanatosError {
    #[cfg(windows)]
    pub fn last_os_error() -> ThanatosError {
        use windows::Win32::Foundation::GetLastError;

        ThanatosError::WinError(unsafe { GetLastError().to_hresult() })
    }
}
