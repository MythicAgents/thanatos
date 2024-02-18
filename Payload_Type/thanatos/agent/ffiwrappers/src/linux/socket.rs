use std::{marker::PhantomData, ptr::NonNull};

#[repr(i32)]
#[derive(Default)]
pub enum Family {
    AfInet = libc::AF_INET,
    AfInet6 = libc::AF_INET6,
    #[default]
    Unspec = libc::AF_UNSPEC,
    Other(i32),
}

impl From<i32> for Family {
    fn from(value: i32) -> Self {
        match value {
            libc::AF_INET => Self::AfInet,
            libc::AF_INET6 => Self::AfInet6,
            libc::AF_UNSPEC => Self::Unspec,
            _ => Self::Other(value),
        }
    }
}

impl From<Family> for i32 {
    fn from(value: Family) -> Self {
        match value {
            Family::AfInet => libc::AF_INET,
            Family::AfInet6 => libc::AF_INET6,
            Family::Unspec => libc::AF_UNSPEC,
            Family::Other(v) => v,
        }
    }
}

impl From<u16> for Family {
    fn from(value: u16) -> Self {
        (value as i32).into()
    }
}

impl From<Family> for u16 {
    fn from(value: Family) -> Self {
        value.into()
    }
}

#[repr(i32)]
#[derive(Default)]
pub enum SockType {
    #[default]
    Any = 0,
    SockStream = libc::SOCK_STREAM,
    SockDgram = libc::SOCK_DGRAM,
    SockSeqPacket = libc::SOCK_SEQPACKET,
    SockRaw = libc::SOCK_RAW,
    SockRdm = libc::SOCK_RDM,
    SockPacket = libc::SOCK_PACKET,
}

impl From<i32> for SockType {
    fn from(value: i32) -> Self {
        match value {
            libc::SOCK_STREAM => Self::SockStream,
            libc::SOCK_DGRAM => Self::SockDgram,
            libc::SOCK_SEQPACKET => Self::SockSeqPacket,
            libc::SOCK_RAW => Self::SockRaw,
            libc::SOCK_RDM => Self::SockRdm,
            libc::SOCK_PACKET => Self::SockPacket,
            _ => Self::Any,
        }
    }
}

pub struct AfInet;
pub struct AfInet6;

pub trait SockAddrFamily {
    type Inner;
}

impl SockAddrFamily for AfInet {
    type Inner = libc::sockaddr_in;
}

impl SockAddrFamily for AfInet6 {
    type Inner = libc::sockaddr_in6;
}

pub enum SockAddr<'a> {
    AfInet(SockAddrIn<'a, AfInet>),
    AfInet6(SockAddrIn<'a, AfInet6>),
}

#[repr(transparent)]
pub struct SockAddrIn<'a, T: SockAddrFamily> {
    addr: NonNull<T::Inner>,
    _marker: PhantomData<&'a T::Inner>,
}

impl<'a> SockAddrIn<'a, AfInet> {
    pub fn sin_family(&self) -> Family {
        unsafe { self.addr.as_ref().sin_family }.into()
    }

    pub fn sin_port(&self) -> u16 {
        unsafe { self.addr.as_ref().sin_port }
    }

    pub fn sin_addr(&self) -> &libc::in_addr {
        unsafe { &self.addr.as_ref().sin_addr }
    }
}
