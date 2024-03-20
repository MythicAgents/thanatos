use std::{cell::UnsafeCell, ffi::c_void, marker::PhantomData, ptr::NonNull};

use windows::Win32::{
    Foundation::{ERROR_BUFFER_OVERFLOW, ERROR_SUCCESS},
    NetworkManagement::IpHelper::{
        GetAdaptersAddresses, GAA_FLAG_INCLUDE_ALL_INTERFACES, GAA_FLAG_INCLUDE_GATEWAYS,
        GAA_FLAG_INCLUDE_PREFIX, GAA_FLAG_INCLUDE_TUNNEL_BINDINGORDER, GAA_FLAG_SKIP_ANYCAST,
        GAA_FLAG_SKIP_MULTICAST, GAA_FLAG_SKIP_UNICAST, GET_ADAPTERS_ADDRESSES_FLAGS,
        IP_ADAPTER_ADDRESSES_LH,
    },
};

use crate::errors::FfiError;

use super::{AdapterFamily, UnicastAddress};

bitflags::bitflags! {
    pub struct AdapterFlags: u32 {
        const SKIP_UNICAST = GAA_FLAG_SKIP_UNICAST.0;
        const SKIP_ANYCAST = GAA_FLAG_SKIP_ANYCAST.0;
        const SKIP_MULTICAST = GAA_FLAG_SKIP_MULTICAST.0;
        const INCLUDE_PREFIX = GAA_FLAG_INCLUDE_PREFIX.0;
        const INCLUDE_GATEWAYS = GAA_FLAG_INCLUDE_GATEWAYS.0;
        const INCLUDE_ALL_INTERFACES = GAA_FLAG_INCLUDE_ALL_INTERFACES.0;
        const INCLUDE_TUNNEL_BINDINGORDER = GAA_FLAG_INCLUDE_TUNNEL_BINDINGORDER.0;
    }
}

pub struct IpAdaptersList<IpvType: AdapterFamily> {
    backing: UnsafeCell<Vec<u8>>,
    _family: PhantomData<IpvType>,
}

impl<IpvType: AdapterFamily> IpAdaptersList<IpvType> {
    pub fn addresses() -> Result<IpAdaptersList<IpvType>, FfiError> {
        Self::with_flags(None)
    }
    pub fn with_flags(flags: Option<AdapterFlags>) -> Result<IpAdaptersList<IpvType>, FfiError> {
        let flags = GET_ADAPTERS_ADDRESSES_FLAGS(flags.map(|f| f.bits()).unwrap_or(0));

        let mut size_required = 0;

        let ret = unsafe {
            GetAdaptersAddresses(IpvType::VALUE.into(), flags, None, None, &mut size_required)
        };

        if ret != ERROR_BUFFER_OVERFLOW.0 {
            return Err(FfiError::OsError(ret as i32));
        }

        let mut buffer: Vec<u8> = Vec::with_capacity(size_required as usize);

        let ret = unsafe {
            GetAdaptersAddresses(
                IpvType::VALUE.into(),
                flags,
                None,
                Some(buffer.as_mut_ptr().cast()),
                &mut size_required,
            )
        };

        if ret != ERROR_SUCCESS.0 {
            return Err(FfiError::OsError(ret as i32));
        }

        unsafe { buffer.set_len(size_required as usize) };

        Ok(IpAdaptersList {
            backing: UnsafeCell::new(buffer),
            _family: PhantomData,
        })
    }

    pub fn first(&self) -> IpAdapter<IpvType> {
        IpAdapter {
            adapter: unsafe { NonNull::new_unchecked((*self.backing.get()).as_mut_ptr().cast()) },
            _marker: PhantomData,
            _family: PhantomData,
        }
    }

    pub fn iter(&self) -> IpAdaptersListIterator<IpvType> {
        IpAdaptersListIterator {
            adapter: unsafe { (*self.backing.get()).as_mut_ptr().cast() },
            _marker: PhantomData,
            _family: PhantomData,
        }
    }
}

#[repr(transparent)]
pub struct IpAdaptersListIterator<'a, IpvType: AdapterFamily> {
    adapter: *mut IP_ADAPTER_ADDRESSES_LH,
    _marker: PhantomData<&'a IP_ADAPTER_ADDRESSES_LH>,
    _family: PhantomData<IpvType>,
}

impl<'a, IpvType: AdapterFamily> Iterator for IpAdaptersListIterator<'a, IpvType> {
    type Item = IpAdapter<'a, IpvType>;

    fn next(&mut self) -> Option<Self::Item> {
        let adapter = NonNull::new(self.adapter)?;
        self.adapter = unsafe { adapter.as_ref() }.Next;

        Some(IpAdapter {
            adapter,
            _marker: PhantomData,
            _family: PhantomData,
        })
    }
}

#[repr(transparent)]
pub struct IpAdapter<'a, IpvType: AdapterFamily> {
    adapter: NonNull<IP_ADAPTER_ADDRESSES_LH>,
    _marker: PhantomData<&'a IP_ADAPTER_ADDRESSES_LH>,
    _family: PhantomData<IpvType>,
}

impl<'a, IpvType: AdapterFamily> IpAdapter<'_, IpvType> {
    pub const fn if_index(&self) -> u32 {
        unsafe { self.adapter.as_ref() }.IfType
    }

    pub fn adapter_name(&self) -> Result<String, FfiError> {
        unsafe { self.adapter.as_ref().AdapterName.to_string() }
            .map_err(|_| FfiError::StringParseError)
    }

    pub fn first_unicast_address(&self) -> Option<UnicastAddress<IpvType>> {
        let unicast_address = NonNull::new(unsafe { self.adapter.as_ref() }.FirstUnicastAddress)?;
        Some(unsafe { UnicastAddress::from_raw(unicast_address) })
    }

    pub const fn first_anycast_address(&self) -> *const c_void {
        unsafe { self.adapter.as_ref() }.FirstAnycastAddress.cast()
    }

    pub const fn first_multicast_address(&self) -> *const c_void {
        unsafe { self.adapter.as_ref() }
            .FirstMulticastAddress
            .cast()
    }

    pub fn dns_suffix(&self) -> Result<String, FfiError> {
        unsafe { self.adapter.as_ref().DnsSuffix.to_string() }
            .map_err(|_| FfiError::StringParseError)
    }

    pub fn description(&self) -> Result<String, FfiError> {
        unsafe { self.adapter.as_ref().Description.to_string() }
            .map_err(|_| FfiError::StringParseError)
    }

    pub fn friendly_name(&self) -> Result<String, FfiError> {
        unsafe { self.adapter.as_ref().FriendlyName.to_string() }
            .map_err(|_| FfiError::StringParseError)
    }
}

#[cfg(test)]
mod tests {
    use super::IpAdaptersList;
    use crate::socket::AfUnspec;

    #[test]
    fn adapters() {
        let interfaces: IpAdaptersList<AfUnspec> = IpAdaptersList::addresses().unwrap();

        for interface in interfaces.iter() {}
    }
}
