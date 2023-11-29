//! Errors for the whole project.
//! Every function which returns an error should map to a `ThanatosError`.
//! This is for consistency and also to reduce the binary sizes.
//! External libraries are designed to have very verbose error information to make debugging easier.
//! This verbose debugging info includes a lot of bloat and embedded strings which are not ideal for
//! offensive tooling.
//!
//! ALL errors should be handled correctly and ergonomically. No uses of `.unwrap()` or `.expect()`!
//! These will severely bloat the code and cause unexpected panics. Panics == bad

use std::io;

/// Generic error type signifying if the C2 profile had an internal error or if
/// the error was that the profile could not connect. Each type is handled differently.
#[derive(Debug)]
pub enum ProfileInternalError {
    /// C2 Profile experienced a fatal error.
    Fatal,

    /// C2 Profile could not connect.
    ConnectError,

    /// P2P Profile could not bind.
    BindError,

    /// P2P profile received no connection.
    NoConnection,
}

impl ToString for ProfileInternalError {
    fn to_string(&self) -> String {
        format!("{:?}", self)
    }
}

/// Error types for the project.
#[derive(Debug)]
pub enum ThanatosError {
    /// String could not be parsed correctly.
    StringParseError,

    /// Data could not be decoded into JSON.
    JsonDecodeError,

    /// Data could not be encoded into JSON.
    JsonEncodeError,

    /// Data could not be base64 decoded.
    Base64DecodeError,

    /// Mythic returned an "error" in the status field
    MythicStatusError,

    /// Generic OS error. Wraps a Linux `errno` or a Windows `GetLastError()`.
    OsError {
        /// The OS error code.
        code: i32,

        /// The OS error message.
        msg: String,
    },

    /// Error for a C2 Profile.
    ProfileError(ProfileInternalError),

    /// Failed to read data from p2p connection.
    ConnectionReadError,

    /// Failed to write data to the p2p connection.
    ConnectionWriteError,

    /// Failed to link to the p2p agent.
    LinkConnectError,

    /// Invalid handle value
    InvalidHandle,

    /// Callback host is malformed
    MalformedCallbackHost,

    /// Could not open process token
    ProcessTokenOpenError,

    /// Failed to set the read timeout on a tcp stream
    SetReadTimeoutError,

    /// Failed to set the write timeout on a tcp stream
    SetWriteTimeoutError,

    /// Failed to AES encrypt data
    AesEncryptError,

    /// Failed to AES decrypt data
    AesDecryptError,

    /// Failed to calculate the HMAC for the data
    CalcHmacError,

    /// Failed to sha256 hash data
    Sha256HashError,

    /// Message has a malformed signature
    MessageSignatureMismatch,

    /// Failed to generate RSA private key for EKE
    RsaKeyGenerateError,

    /// Failed to decrypt the AES key from the EKE
    RsaDecryptError,

    /// AES key is not 32 bytes
    AesKeyTruncateError,

    /// No profiles configured in the agent (shouldn't happen)
    NoProfiles,

    /// Command already loaded.
    CommandLoadedError,

    /// Could not listen at the following path
    PipeListen,

    /// Failed to join thread
    ThreadJoinError,

    /// Spawnto value not set
    NoSpawnTo,
}

impl ThanatosError {
    /// Creates a `ThanatosError` out of the last OS error code generated for Windows.
    #[cfg(target_os = "windows")]
    pub fn os_error() -> Self {
        let code = io::Error::last_os_error().raw_os_error().unwrap_or(-1);
        let msg = if code != -1 {
            windows::core::Error::from_win32().message().to_string()
        } else {
            "Unknown Error Code".to_string()
        };

        Self::OsError { code, msg }
    }

    /// Creates a `ThanatosError` out of the last OS error code generated for Linux.
    #[cfg(target_os = "linux")]
    pub fn os_error() -> Self {
        let code = io::Error::last_os_error().raw_os_error().unwrap_or(-1);
        let msg = if code != -1 {
            unsafe { std::ffi::CStr::from_ptr(libc::strerror(code)) }
                .to_string_lossy()
                .to_string()
        } else {
            "Unknown Error Code".to_string()
        };

        Self::OsError { code, msg }
    }

    /// Converts an integer error code into a `ThanatosError`.
    #[cfg(target_os = "windows")]
    pub fn from_error_code(code: i32) -> Self {
        let e = windows::core::HRESULT(code);
        let msg = windows::core::Error::from(e).message().to_string();

        Self::OsError { code, msg }
    }

    /// Converts an integer error code into a `ThanatosError`.
    #[cfg(target_os = "linux")]
    pub fn from_error_code(code: i32) -> Self {
        let msg = unsafe { std::ffi::CStr::from_ptr(libc::strerror(code)) }
            .to_string_lossy()
            .to_string();

        Self::OsError { code, msg }
    }
}

impl ToString for ThanatosError {
    fn to_string(&self) -> String {
        match self {
            Self::OsError { code, msg } => format!("{0} (0x{0:x}) - {1}", code, msg),
            _ => format!("{:?}", self),
        }
    }
}
