//! Error types for the crate

/// Error type
#[derive(Debug)]
pub enum ThanatosLoadAPIError {
    /// Task ID is not set properly in the environment variables.
    NoTaskIDSet,

    /// No send pipe fd found in the environment variables.
    NoSendPipe,

    /// Generic IO error.
    IOError(std::io::Error),

    /// Failed to write data along the pipe.
    DataWriteError,
}
