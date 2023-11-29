use std::{
    io::{ErrorKind, Read},
    net::TcpStream,
    time::Duration,
};

use agent_utils::{crypto::b64encode, errors::ThanatosError};
use std::{
    io::{self, Read, Write},
    time::Duration,
};

/// Trait encompassing all the functionality of an IOHandle
pub trait ReadWritePoll: Write + PollReader {}

/// Trait for allowing polling input on a `Read` type
pub trait PollReader: Read {
    /// Waits for a specified duration until the reader can be read from
    fn check_readable(&mut self, timeout: Duration) -> bool;
}

/// IOHandle for sending and receiving data along p2p profiles
pub struct IOHandle<T: ReadWritePoll>(T);

impl<T: ReadWritePoll> IOHandle<T> {
    pub fn new(v: T) -> IOHandle<T> {
        Self(v)
    }
}

impl<T: ReadWritePoll> Read for IOHandle<T> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.0.read(buf)
    }
}

impl<T: ReadWritePoll> Write for IOHandle<T> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.0.flush()
    }
}

/// Trait defining a P2P profile type.
pub trait P2PProfile<T: ReadWritePoll> {
    /// Checks if there is a new P2P connection and returns a handle to it
    fn poll_connection(
        &mut self,
        timeout: std::time::Duration,
    ) -> Result<IOHandle<T>, ThanatosError>;
}

impl<T: ReadWritePoll> PollReader for IOHandle<T> {
    fn check_readable(&mut self, timeout: std::time::Duration) -> bool {
        self.0.check_readable(timeout)
    }
}

#[cfg(feature = "tcp")]
impl ReadWritePoll for TcpStream {}

#[cfg(feature = "tcp")]
impl From<TcpStream> for IOHandle<TcpStream> {
    fn from(value: TcpStream) -> Self {
        IOHandle::new(value)
    }
}
