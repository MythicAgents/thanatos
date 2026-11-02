use std::net::UdpSocket;

#[cfg_attr(unix, path = "platform/unix.rs")]
#[cfg_attr(windows, path = "platform/windows.rs")]
mod imp;

use serde_repr::{Deserialize_repr, Serialize_repr};

pub use self::imp::*;

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize_repr, Deserialize_repr,
)]
#[repr(u8)]
pub enum IntegrityLevel {
    Low = 1,
    Medium = 2,
    High = 3,
    System = 4,
}

/// Get the local IP address of the machine
pub fn local_ipaddress() -> Option<String> {
    let socket = UdpSocket::bind("0.0.0.0:0").ok()?;
    socket.connect("8.8.8.8:80").ok()?;
    Some(socket.local_addr().ok()?.ip().to_string())
}
