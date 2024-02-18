use std::{ffi::CStr, marker::PhantomData, ptr::NonNull};

use crate::errors::{EaiError, FfiError};
use bitflags::bitflags;

use super::socket::{Family, SockType};

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
    pub const fn as_i32(&self) -> i32 {
        self.bits()
    }
}

pub struct Hints {
    pub family: Family,
    pub socktype: SockType,
    pub flags: AiFlags,
}

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
            hints_data.ai_family = h.family.as_i32();
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

    pub const fn first(&self) -> AddrInfo {
        AddrInfo {
            addrinfo: self.addrinfo,
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
pub struct AddrInfo<'a> {
    addrinfo: NonNull<libc::addrinfo>,
    _marker: PhantomData<&'a libc::addrinfo>,
}

impl<'a> AddrInfo<'a> {
    pub const fn ai_flags(&self) -> i32 {
        unsafe { self.addrinfo.as_ref().ai_flags }
    }

    pub const fn ai_family(&self) -> Family {
        Family::from_value(unsafe { self.addrinfo.as_ref().ai_family })
    }

    pub const fn ai_socktype(&self) -> SockType {
        SockType::from_value(unsafe { self.addrinfo.as_ref().ai_socktype })
    }

    pub const fn ai_protocol(&self) -> i32 {
        unsafe { self.addrinfo.as_ref().ai_protocol }
    }

    pub fn canonname(&self) -> &str {
        unsafe {
            CStr::from_ptr(self.addrinfo.as_ref().ai_canonname)
                .to_str()
                .unwrap_unchecked()
        }
    }
}

impl<'a> From<NonNull<libc::addrinfo>> for AddrInfo<'a> {
    fn from(value: NonNull<libc::addrinfo>) -> Self {
        AddrInfo {
            addrinfo: value,
            _marker: PhantomData,
        }
    }
}

impl<'a> Iterator for AddrInfo<'a> {
    type Item = AddrInfo<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.addrinfo = NonNull::new(unsafe { self.addrinfo.as_ref().ai_next })?;
        Some(self.addrinfo.into())
    }
}
