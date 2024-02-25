use std::{
    ffi::{CStr, CString},
    marker::PhantomData,
    ptr::NonNull,
};

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
    pub fn new() -> Result<AddrInfoList, FfiError> {
        let addr = CString::new("0.0.0.0").map_err(|_| FfiError::InteriorNull)?;
        Self::with_opts(
            Some(&addr),
            None,
            Some(Hints {
                socktype: SockType::Any,
                flags: AiFlags::ALL | AiFlags::V4MAPPED,
                family: Family::Unspec,
            }),
        )
    }

    pub fn with_nodename(node: &str) -> Result<AddrInfoList, FfiError> {
        let nodename = CString::new(node).map_err(|_| FfiError::InteriorNull)?;
        Self::with_opts(
            Some(&nodename),
            None,
            Some(Hints {
                socktype: SockType::Any,
                flags: AiFlags::ALL | AiFlags::V4MAPPED,
                family: Family::Unspec,
            }),
        )
    }

    pub fn with_hints(hints: Hints) -> Result<AddrInfoList, FfiError> {
        let addr = CString::new("0.0.0.0").map_err(|_| FfiError::InteriorNull)?;
        Self::with_opts(Some(&addr), None, Some(hints))
    }

    pub fn with_opts(
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

    pub const fn first(&self) -> AddrInfoEntry {
        AddrInfoEntry {
            addrinfo: self.addrinfo,
            _marker: PhantomData,
        }
    }

    pub const fn iter(&self) -> AddrInfoListIterator {
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
    type Item = AddrInfoEntry<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let addrinfo = NonNull::new(self.addrinfo)?;
        self.addrinfo = unsafe { addrinfo.as_ref().ai_next };

        Some(AddrInfoEntry {
            addrinfo,
            _marker: PhantomData,
        })
    }
}

#[repr(transparent)]
pub struct AddrInfoEntry<'a> {
    addrinfo: NonNull<libc::addrinfo>,
    _marker: PhantomData<&'a libc::addrinfo>,
}

impl<'a> AddrInfoEntry<'a> {
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

    pub fn canonname(&self) -> Result<&str, FfiError> {
        let name = unsafe { self.addrinfo.as_ref().ai_canonname };
        if name.is_null() {
            Err(FfiError::CanonNameNotFound)
        } else {
            unsafe { CStr::from_ptr(name).to_str() }.map_err(|_| FfiError::CanonNameNotFound)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::AddrInfoList;

    #[test]
    fn iter_test() {
        let interfaces = AddrInfoList::new().unwrap();

        let first_interface = interfaces.first();
        let first_iter_interface = interfaces.iter().next().unwrap();

        assert_eq!(first_interface.ai_flags(), first_iter_interface.ai_flags());

        assert_eq!(
            first_interface.ai_family(),
            first_iter_interface.ai_family()
        );

        assert_eq!(
            first_interface.ai_socktype(),
            first_iter_interface.ai_socktype()
        );

        assert_eq!(
            first_interface.ai_protocol(),
            first_iter_interface.ai_protocol()
        );
    }
}
