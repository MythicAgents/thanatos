use std::{marker::PhantomData, ptr::NonNull};

use crate::internal::SealedTrait;

pub struct AfUnspec;
impl SealedTrait for AfUnspec {}

pub enum SockAddr<'a> {
    AfInet(SockAddrIn<'a, AfInet>),
    AfInet6(SockAddrIn<'a, AfInet6>),
}

impl<'a> SockAddr<'a> {
    pub(crate) unsafe fn from_ptr(s: *mut libc::sockaddr) -> Option<SockAddr<'a>> {
        match (*s).sa_family as i32 {
            libc::AF_INET6 => Some(SockAddr::AfInet6(SockAddrIn::<AfInet6>::from_raw(
                NonNull::new(s)?.cast(),
            ))),
            libc::AF_INET => Some(SockAddr::AfInet(SockAddrIn::<AfInet>::from_raw(
                NonNull::new(s)?.cast(),
            ))),
            _ => None,
        }
    }
}

#[repr(transparent)]
pub struct SockAddrIn<'a, S: SockAddrFamily> {
    pub(crate) addr: NonNull<S::Inner>,
    _marker: PhantomData<&'a S::Inner>,
}

impl<'a, S: SockAddrFamily> SockAddrIn<'a, S> {
    pub(crate) unsafe fn from_raw(addr: NonNull<S::Inner>) -> SockAddrIn<'a, S> {
        SockAddrIn {
            addr,
            _marker: PhantomData,
        }
    }
}

#[repr(i32)]
#[derive(Default, Debug, PartialEq, Eq)]
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
#[derive(Default, Debug, PartialEq, Eq)]
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

impl<'a> SockAddrIn<'a, AfInet> {
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

#[cfg(test)]
mod tests {
    use crate::linux::ifaddrs::{IfAddrsList, IfuAddr};

    use super::{Family, SockAddr, SockType};

    #[test]
    #[ignore = "Pending rewrite"]
    fn family() {
        let mappings = [
            (libc::AF_INET, Family::AfInet),
            (libc::AF_INET6, Family::AfInet6),
            (libc::AF_UNSPEC, Family::Unspec),
            (1, Family::Other(1)),
        ];

        for (afval, fam) in mappings {
            assert_eq!(fam, Family::from_value(afval));
            assert_eq!(afval, fam.as_i32());
        }
    }

    #[test]
    #[ignore = "Pending rewrite"]
    fn socktype() {
        let mappings = [
            (libc::SOCK_STREAM, SockType::SockStream),
            (libc::SOCK_DGRAM, SockType::SockDgram),
            (libc::SOCK_SEQPACKET, SockType::SockSeqPacket),
            (libc::SOCK_RAW, SockType::SockRaw),
            (libc::SOCK_RDM, SockType::SockRdm),
            (libc::SOCK_PACKET, SockType::SockPacket),
            (0, SockType::Any),
        ];

        for (sockval, socktype) in mappings {
            assert_eq!(socktype, SockType::from_value(sockval));
        }
    }

    // This test doesn't really do anything on its own. Its main purpose is for using
    // sanitizers to check for out of bounds memory accesses, uninitialized reads and
    // memory leaks.
    #[test]
    #[ignore = "Pending rewrite"]
    fn accessor_checks() {
        let interfaces = IfAddrsList::new().unwrap();

        for iface in interfaces.iter() {
            if let Some(ifaaddr) = iface.ifa_addr() {
                match ifaaddr {
                    SockAddr::AfInet(inet) => {
                        let _ = inet.sin_addr().s_addr;
                        let _ = inet.sin_family();
                        let _ = inet.sin_port();
                    }
                    SockAddr::AfInet6(inet6) => {
                        let _ = inet6.sin6_addr().s6_addr;
                        let _ = inet6.sin6_family();
                        let _ = inet6.sin6_flowinfo();
                        let _ = inet6.sin6_port();
                        let _ = inet6.sin6_scope_id();
                    }
                }
            }

            if let Some(ifaifu) = iface.ifa_ifu() {
                match ifaifu {
                    IfuAddr::Broadcast(broadcast) => match broadcast {
                        SockAddr::AfInet(inet) => {
                            let _ = inet.sin_addr().s_addr;
                            let _ = inet.sin_family();
                            let _ = inet.sin_port();
                        }
                        SockAddr::AfInet6(inet6) => {
                            let _ = inet6.sin6_addr().s6_addr;
                            let _ = inet6.sin6_family();
                            let _ = inet6.sin6_flowinfo();
                            let _ = inet6.sin6_port();
                            let _ = inet6.sin6_scope_id();
                        }
                    },

                    IfuAddr::PointToPointDst(dest) => match dest {
                        SockAddr::AfInet(inet) => {
                            let _ = inet.sin_addr().s_addr;
                            let _ = inet.sin_family();
                            let _ = inet.sin_port();
                        }
                        SockAddr::AfInet6(inet6) => {
                            let _ = inet6.sin6_addr().s6_addr;
                            let _ = inet6.sin6_family();
                            let _ = inet6.sin6_flowinfo();
                            let _ = inet6.sin6_port();
                            let _ = inet6.sin6_scope_id();
                        }
                    },
                }
            }
        }
    }
}
