use std::{ffi::CStr, marker::PhantomData, ptr::NonNull};

use crate::errors::FfiError;

pub struct IfAddrsList {
    ifaddrs: NonNull<libc::ifaddrs>,
    _marker: PhantomData<libc::addrinfo>,
}

impl IfAddrsList {
    pub fn new() -> Result<IfAddrsList, FfiError> {
        let mut ifap = std::ptr::null_mut();

        if unsafe { libc::getifaddrs(&mut ifap) } != 0 {
            return Err(FfiError::os_error());
        }

        Ok(IfAddrsList {
            ifaddrs: NonNull::new(ifap).ok_or(FfiError::os_error())?,
            _marker: PhantomData,
        })
    }

    pub fn first<'a>(&'a self) -> IfAddr<'a> {
        self.ifaddrs.into()
    }
}

impl Drop for IfAddrsList {
    fn drop(&mut self) {
        unsafe { libc::freeifaddrs(self.ifaddrs.as_ptr()) };
    }
}

pub struct IfAddr<'a> {
    ifaddr: NonNull<libc::ifaddrs>,
    _marker: PhantomData<&'a libc::ifaddrs>,
}

impl<'a> IfAddr<'a> {
    pub fn name(&self) -> &str {
        unsafe {
            CStr::from_ptr(self.ifaddr.as_ref().ifa_name)
                .to_str()
                .unwrap_unchecked()
        }
    }
}

impl<'a> From<NonNull<libc::ifaddrs>> for IfAddr<'a> {
    fn from(value: NonNull<libc::ifaddrs>) -> Self {
        IfAddr {
            ifaddr: value,
            _marker: PhantomData,
        }
    }
}

impl<'a> Iterator for IfAddr<'a> {
    type Item = IfAddr<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.ifaddr = NonNull::new(unsafe { self.ifaddr.as_ref().ifa_next })?;
        Some(self.ifaddr.into())
    }
}
