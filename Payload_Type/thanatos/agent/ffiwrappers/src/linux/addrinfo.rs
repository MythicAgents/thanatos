use std::{ffi::CStr, marker::PhantomData, ptr::NonNull};

use crate::errors::{EaiError, FfiError};
use bitflags::bitflags;

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

bitflags! {
    pub struct AiFlags: i32 {
        const PASSIVE = libc::AI_PASSIVE;
        const CANONNAME = libc::AI_CANONNAME;
        const NUMERICHOST = libc::AI_NUMERICHOST;
        const V4MAPPED = libc::AI_V4MAPPED;
        const ALL = libc::AI_ALL;
        const ADDRCONFIG = libc::AI_ADDRCONFIG;
    }
}

impl AiFlags {
    pub fn as_i32(&self) -> i32 {
        self.bits()
    }
}

pub struct Hints {
    pub family: Family,
    pub socktype: SockType,
    pub flags: AiFlags,
}

#[repr(transparent)]
pub struct AddrInfoList {
    addrinfo: NonNull<libc::addrinfo>,
    _marker: PhantomData<libc::addrinfo>,
}

impl AddrInfoList {
    pub fn new(
        node: Option<&CStr>,
        service: Option<&CStr>,
        hints: Option<Hints>,
    ) -> Result<AddrInfoList, FfiError> {
        let node_ptr = node.map(|n| n.as_ptr()).unwrap_or_else(std::ptr::null);
        let service_ptr = service.map(|s| s.as_ptr()).unwrap_or_else(std::ptr::null);

        let mut hints_data: libc::addrinfo = unsafe { std::mem::zeroed() };
        let hints_ptr = if let Some(h) = hints {
            hints_data.ai_family = h.family.into();
            hints_data.ai_socktype = h.socktype as i32;
            hints_data.ai_flags = h.flags.as_i32();

            &hints_data
        } else {
            std::ptr::null()
        };

        let mut res = std::ptr::null_mut();

        let ret = unsafe { libc::getaddrinfo(node_ptr, service_ptr, hints_ptr, &mut res) };

        if ret != 0 {
            return Err(FfiError::GaiError(EaiError::from_code(ret)));
        }

        Ok(Self {
            addrinfo: NonNull::new(res).ok_or(FfiError::NonNullPointer)?,
            _marker: PhantomData,
        })
    }

    pub fn iter(&self) -> AddrInfoListIterator<'_> {
        AddrInfoListIterator {
            addrinfo: self.addrinfo.as_ptr(),
            _marker: PhantomData,
        }
    }
}

impl Drop for AddrInfoList {
    fn drop(&mut self) {
        unsafe { libc::freeaddrinfo(self.addrinfo.as_ptr()) };
    }
}

#[repr(transparent)]
pub struct AddrInfoListIterator<'a> {
    addrinfo: *mut libc::addrinfo,
    _marker: PhantomData<&'a libc::addrinfo>,
}

impl<'a> Iterator for AddrInfoListIterator<'a> {
    type Item = AddrInfo<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let curr = AddrInfo {
            addrinfo: NonNull::new(self.addrinfo)?,
            _marker: PhantomData,
        };

        self.addrinfo = unsafe { *self.addrinfo }.ai_next;
        Some(curr)
    }
}

#[repr(transparent)]
pub struct AddrInfo<'a> {
    addrinfo: NonNull<libc::addrinfo>,
    _marker: PhantomData<&'a libc::addrinfo>,
}

impl<'a> AddrInfo<'a> {
    pub fn ai_flags(&self) -> i32 {
        unsafe { self.addrinfo.as_ref() }.ai_flags
    }

    pub fn ai_family(&self) -> Family {
        unsafe { self.addrinfo.as_ref() }.ai_family.into()
    }

    pub fn ai_socktype(&self) -> SockType {
        unsafe { self.addrinfo.as_ref() }.ai_socktype.into()
    }

    pub fn ai_protocol(&self) -> i32 {
        unsafe { self.addrinfo.as_ref() }.ai_protocol
    }

    pub fn canonname(&self) -> Option<&CStr> {
        if unsafe { self.addrinfo.as_ref().ai_canonname }.is_null() {
            return None;
        }

        Some(unsafe { CStr::from_ptr(self.addrinfo.as_ref().ai_canonname) })
    }
}
