#![warn(missing_docs)]

//! Library to assist with creating loadable modules for [MythicAgents/thanatos](https://github.com/MythicAgents/tetanus)

#[cfg(feature = "macros")]
pub use thanatos_module_macros::entrypoint;

pub mod errors;
use errors::ThanatosLoadAPIError;

/// Status for sending and receiving tasking.
#[repr(C)]
pub enum MythicStatus {
    /// The sent task has a success status.
    Success = 0,

    /// The sent task has an error status.
    Error = 1,
}

/// Location for sending the data.
#[repr(C)]
pub enum Callback {
    /// Data should be sent to the Mythic task output.
    Output = 0,

    /// Data should be sent to Mythic for further server side processing.
    ProcessResponse = 1,
}

/// Function to send data back to Mythic.
#[cfg(all(target_os = "linux", feature = "std"))]
pub fn send_data(
    status: MythicStatus,
    callback: Callback,
    data: &[u8],
) -> Result<(), ThanatosLoadAPIError> {
    use std::{fs::File, io::Write, os::fd::FromRawFd};

    let sendfd = std::env::var("LD_SENDPIPE")
        .map_err(|_| ThanatosLoadAPIError::NoSendPipe)?
        .parse()
        .map_err(|_| ThanatosLoadAPIError::NoSendPipe)?;
    let mut sender = unsafe { File::from_raw_fd(sendfd) };

    let mut buffer = vec![status as u8, callback as u8];
    buffer.extend_from_slice(data);

    sender
        .write_all(
            &<[u8; 4]>::try_from(&buffer.len().to_le_bytes()[..4])
                .map_err(|_| ThanatosLoadAPIError::DataWriteError)?,
        )
        .map_err(|_| ThanatosLoadAPIError::DataWriteError)?;

    sender
        .write_all(&buffer)
        .map_err(|_| ThanatosLoadAPIError::DataWriteError)
}
