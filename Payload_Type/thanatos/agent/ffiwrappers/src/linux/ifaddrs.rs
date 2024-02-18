use std::{ffi::CStr, marker::PhantomData, ptr::NonNull};

use crate::errors::FfiError;

use super::socket::SockAddr;

bitflags::bitflags! {
    pub struct IffFlags: i32 {
        const UP = libc::IFF_UP;
        const BROADCAST = libc::IFF_BROADCAST;
        const DEBUG = libc::IFF_DEBUG;
        const LOOPBACK = libc::IFF_LOOPBACK;
        const POINTTOPOINT = libc::IFF_POINTOPOINT;
        const RUNNING = libc::IFF_RUNNING;
        const NOARP = libc::IFF_NOARP;
        const PROMISC = libc::IFF_PROMISC;
        const NOTRAILERS = libc::IFF_NOTRAILERS;
        const ALLMULTI = libc::IFF_MULTICAST;
        const MASTER = libc::IFF_MASTER;
        const SLAVE = libc::IFF_SLAVE;
        const MULTICAST = libc::IFF_MULTICAST;
        const PORTSEL = libc::IFF_PORTSEL;
        const AUTOMEDIA = libc::IFF_AUTOMEDIA;
        const DYNAMIC = libc::IFF_DYNAMIC;
        const LOWER_UP = libc::IFF_LOWER_UP;
        const DORMANT = libc::IFF_DORMANT;
        const ECHO = libc::IFF_ECHO;
        const _ = !0;
    }
}

impl IffFlags {
    pub const fn as_i32(&self) -> i32 {
        self.bits()
    }
}

#[repr(transparent)]
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

    pub const fn first(&self) -> IfAddr {
        IfAddr {
            ifaddr: self.ifaddrs,
            _marker: PhantomData,
        }
    }
}

impl Drop for IfAddrsList {
    fn drop(&mut self) {
        unsafe { libc::freeifaddrs(self.ifaddrs.as_ptr()) };
    }
}

pub enum IfuAddr<'a> {
    Broadcast(SockAddr<'a>),
    PointToPointDst(SockAddr<'a>),
}

#[repr(transparent)]
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

    pub const fn ifa_flags(&self) -> IffFlags {
        IffFlags::from_bits_retain(unsafe { self.ifaddr.as_ref().ifa_flags } as i32)
    }

    pub fn ifa_addr(&self) -> Option<SockAddr> {
        unsafe { SockAddr::from_ptr(self.ifaddr.as_ref().ifa_addr) }
    }

    pub fn ifa_netmask(&self) -> Option<SockAddr> {
        unsafe { SockAddr::from_ptr(self.ifaddr.as_ref().ifa_netmask) }
    }

    pub fn ifa_ifu(&self) -> Option<IfuAddr> {
        if self.ifa_flags().contains(IffFlags::BROADCAST) {
            Some(IfuAddr::Broadcast(unsafe {
                SockAddr::from_ptr(self.ifaddr.as_ref().ifa_ifu)?
            }))
        } else if self.ifa_flags().contains(IffFlags::POINTTOPOINT) {
            Some(IfuAddr::PointToPointDst(unsafe {
                SockAddr::from_ptr(self.ifaddr.as_ref().ifa_ifu)?
            }))
        } else {
            None
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
