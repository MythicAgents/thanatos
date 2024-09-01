use crate::profiles::errors::HttpConfigError;

#[derive(Debug)]
pub enum AgentError {
    /// Agent has passed the killdate
    PastKilldate,

    /// Agent could not parse the config
    ConfigParse(ConfigParseError),

    /// Agent passed its retry limit
    RetryLimitReached,

    /// Agent received an unexpected response from Mythic
    UnexpectedResponse,

    /// Session id for encrypted key exchange is incorrect
    SessionidMismatch,

    /// AES key returned from the encrypted key exchange is invalid
    EkeAesKeyInvalid,

    /// Mythic returned an error
    MythicError,
}

#[derive(Debug)]
pub enum ConfigParseError {
    /// The working hours could not be parsed
    WorkingHours,

    /// The working hours start value could not be parsed
    WorkingStart,

    /// The working hours end value could not be parsed
    WorkingEnd,

    /// The AES key is malformed
    AesKey,

    /// Profile config is malformed
    Profile(HttpConfigError),
}
