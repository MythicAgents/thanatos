use std::ptr::NonNull;

use windows::Win32::Networking::WinSock::{
    ADDRESS_FAMILY, AF_INET, AF_INET6, IN_ADDR, SOCKADDR, SOCKADDR_IN, SOCKADDR_IN6,
};

use crate::socket::{AfInet, AfInet6, SockAddr, SockAddrIn};

impl<'a> SockAddr<'a> {
    pub(crate) fn from_raw(addr: NonNull<SOCKADDR>) -> Option<SockAddr<'a>> {
        match unsafe { addr.as_ref().sa_family } {
            AF_INET6 => {
                let addr: NonNull<SOCKADDR_IN6> = addr.cast();
                Some(SockAddr::AfInet6(unsafe {
                    SockAddrIn::<AfInet6>::from_raw(addr)
                }))
            }
            AF_INET => {
                let addr: NonNull<SOCKADDR_IN> = addr.cast();

                Some(SockAddr::AfInet(unsafe {
                    SockAddrIn::<AfInet>::from_raw(addr)
                }))
            }
            _ => None,
        }
    }
}

impl<'a> SockAddrIn<'a, AfInet> {
    pub const fn sin_family(&self) -> ADDRESS_FAMILY {
        unsafe { self.addr.as_ref() }.sin_family
    }

    pub const fn sin_port(&self) -> u16 {
        unsafe { self.addr.as_ref() }.sin_port
    }

    pub const fn sin_addr(&self) -> IN_ADDR {
        unsafe { self.addr.as_ref() }.sin_addr
    }
}
