use std::{marker::PhantomData, ptr::NonNull};

use windows::Win32::NetworkManagement::IpHelper::IP_ADAPTER_UNICAST_ADDRESS_LH;

use crate::socket::{AfUnspec, SockAddr, SockAddrFamily, SockAddrIn};

use super::AdapterFamily;

#[repr(transparent)]
pub struct UnicastAddress<'a, IpvType: AdapterFamily> {
    address: NonNull<IP_ADAPTER_UNICAST_ADDRESS_LH>,
    _marker: PhantomData<&'a IP_ADAPTER_UNICAST_ADDRESS_LH>,
    _family: PhantomData<IpvType>,
}

impl<'a, IpvType: AdapterFamily> UnicastAddress<'_, IpvType> {
    pub(crate) unsafe fn from_raw(
        ptr: NonNull<IP_ADAPTER_UNICAST_ADDRESS_LH>,
    ) -> UnicastAddress<'a, IpvType> {
        UnicastAddress {
            address: ptr,
            _marker: PhantomData,
            _family: PhantomData,
        }
    }
}

impl<'a, IpvType: AdapterFamily + SockAddrFamily> UnicastAddress<'_, IpvType> {
    pub fn address(&self) -> Option<SockAddrIn<IpvType>> {
        Some(unsafe {
            SockAddrIn::from_raw(NonNull::new(self.address.as_ref().Address.lpSockaddr)?.cast())
        })
    }
}

impl<'a> UnicastAddress<'a, AfUnspec> {
    pub fn address(&self) -> Option<SockAddr> {
        SockAddr::from_raw(unsafe { NonNull::new(self.address.as_ref().Address.lpSockaddr)? })
    }
}

impl<'a, IpvType: AdapterFamily> Iterator for UnicastAddress<'a, IpvType> {
    type Item = UnicastAddress<'a, IpvType>;

    fn next(&mut self) -> Option<Self::Item> {
        let next_address = NonNull::new(unsafe { self.address.as_ref() }.Next)?;
        self.address = next_address;

        Some(UnicastAddress {
            address: next_address,
            _marker: PhantomData,
            _family: PhantomData,
        })
    }
}
