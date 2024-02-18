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

impl Family {
    pub const fn from_value(value: i32) -> Family {
        match value {
            libc::AF_INET => Self::AfInet,
            libc::AF_INET6 => Self::AfInet6,
            libc::AF_UNSPEC => Self::Unspec,
            _ => Self::Other(value),
        }
    }

    pub const fn as_i32(&self) -> i32 {
        match self {
            Self::AfInet => libc::AF_INET,
            Self::AfInet6 => libc::AF_INET6,
            Self::Unspec => libc::AF_UNSPEC,
            Self::Other(v) => *v,
        }
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

impl SockType {
    pub const fn from_value(value: i32) -> SockType {
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

impl<'a> SockAddr<'a> {
    /// # Safety
    ///
    /// The pointer is not validated to ensure that it is of the correct type
    pub unsafe fn from_ptr(ptr: *mut libc::sockaddr) -> Option<SockAddr<'a>> {
        if ptr.is_null() {
            return None;
        }

        match unsafe { (*ptr).sa_family } as i32 {
            libc::AF_INET6 => {
                let addr: NonNull<libc::sockaddr_in6> = NonNull::new(ptr.cast())?;
                Some(SockAddr::AfInet6(unsafe {
                    SockAddrIn::<AfInet6>::from_raw(addr)
                }))
            }
            libc::AF_INET => {
                let addr: NonNull<libc::sockaddr_in> = NonNull::new(ptr.cast())?;

                Some(SockAddr::AfInet(unsafe {
                    SockAddrIn::<AfInet>::from_raw(addr)
                }))
            }
            _ => None,
        }
    }
}

#[repr(transparent)]
pub struct SockAddrIn<'a, T: SockAddrFamily> {
    addr: NonNull<T::Inner>,
    _marker: PhantomData<&'a T::Inner>,
}

impl<'a> SockAddrIn<'a, AfInet> {
    /// # Safety
    ///
    /// This pointer is not validated to ensure that it is of the correct type
    pub unsafe fn from_raw(addr: NonNull<libc::sockaddr_in>) -> Self {
        Self {
            addr,
            _marker: PhantomData,
        }
    }

    pub const fn sin_family(&self) -> Family {
        Family::from_value(unsafe { self.addr.as_ref().sin_family } as i32)
    }

    pub fn sin_port(&self) -> u16 {
        unsafe { self.addr.as_ref().sin_port }
    }

    pub fn sin_addr(&self) -> &libc::in_addr {
        unsafe { &self.addr.as_ref().sin_addr }
    }
}

impl<'a> SockAddrIn<'a, AfInet6> {
    /// # Safety
    ///
    /// This pointer is not validated to ensure that it is of the correct type
    pub const unsafe fn from_raw(addr: NonNull<libc::sockaddr_in6>) -> Self {
        Self {
            addr,
            _marker: PhantomData,
        }
    }

    pub fn sin6_family(&self) -> Family {
        Family::from_value(unsafe { self.addr.as_ref().sin6_family } as i32)
    }

    pub fn sin6_port(&self) -> u16 {
        unsafe { self.addr.as_ref().sin6_port }
    }

    pub fn sin6_flowinfo(&self) -> u32 {
        unsafe { self.addr.as_ref().sin6_flowinfo }
    }

    pub fn sin6_addr(&self) -> &libc::in6_addr {
        unsafe { &self.addr.as_ref().sin6_addr }
    }

    pub fn sin6_scope_id(&self) -> u32 {
        unsafe { self.addr.as_ref().sin6_scope_id }
    }
}
