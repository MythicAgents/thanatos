use std::{marker::PhantomData, ptr::NonNull};

use windows::{
    core::Error as WinError,
    Win32::{
        Foundation::{ERROR_BUFFER_OVERFLOW, WIN32_ERROR},
        NetworkManagement::{
            IpHelper::{
                GetAdaptersAddresses, GAA_FLAG_SKIP_ANYCAST, GAA_FLAG_SKIP_DNS_SERVER,
                GAA_FLAG_SKIP_FRIENDLY_NAME, GAA_FLAG_SKIP_MULTICAST, GET_ADAPTERS_ADDRESSES_FLAGS,
                IP_ADAPTER_ADDRESSES_LH, IP_ADAPTER_ADDRESS_DNS_ELIGIBLE,
                IP_ADAPTER_ADDRESS_TRANSIENT, IP_ADAPTER_UNICAST_ADDRESS_LH,
            },
            Ndis::{
                IfOperStatusDormant, IfOperStatusDown, IfOperStatusLowerLayerDown,
                IfOperStatusNotPresent, IfOperStatusTesting, IfOperStatusUnknown, IfOperStatusUp,
                IF_OPER_STATUS,
            },
        },
        Networking::WinSock::{inet_ntop, AF_INET, AF_INET6, AF_UNSPEC, SOCKADDR_IN, SOCKADDR_IN6},
    },
};

fn adapter_flags() -> GET_ADAPTERS_ADDRESSES_FLAGS {
    GAA_FLAG_SKIP_ANYCAST
        | GAA_FLAG_SKIP_MULTICAST
        | GAA_FLAG_SKIP_DNS_SERVER
        | GAA_FLAG_SKIP_FRIENDLY_NAME
}

#[repr(i32)]
#[derive(PartialEq)]
enum IfOperStatus {
    Up = IfOperStatusUp.0,
    Down = IfOperStatusDown.0,
    Testing = IfOperStatusTesting.0,
    Unknown = IfOperStatusUnknown.0,
    Dormant = IfOperStatusDormant.0,
    NotPresent = IfOperStatusNotPresent.0,
    LowerLayerDown = IfOperStatusLowerLayerDown.0,
    Other(i32),
}

impl Into<IfOperStatus> for IF_OPER_STATUS {
    #[allow(non_upper_case_globals)]
    fn into(self) -> IfOperStatus {
        match self {
            IfOperStatusUp => IfOperStatus::Up,
            IfOperStatusDown => IfOperStatus::Down,
            IfOperStatusTesting => IfOperStatus::Testing,
            IfOperStatusUnknown => IfOperStatus::Unknown,
            IfOperStatusDormant => IfOperStatus::Dormant,
            IfOperStatusNotPresent => IfOperStatus::NotPresent,
            IfOperStatusLowerLayerDown => IfOperStatus::LowerLayerDown,
            o => IfOperStatus::Other(o.0),
        }
    }
}

#[repr(u32)]
#[derive(Debug, PartialEq)]
enum UnicastAddressFlags {
    DnsEligible = IP_ADAPTER_ADDRESS_DNS_ELIGIBLE,
    Transient = IP_ADAPTER_ADDRESS_TRANSIENT,
    Other(u32),
    NoFlags = 0,
}

impl Into<UnicastAddressFlags> for u32 {
    fn into(self) -> UnicastAddressFlags {
        match self {
            IP_ADAPTER_ADDRESS_DNS_ELIGIBLE => UnicastAddressFlags::DnsEligible,
            IP_ADAPTER_ADDRESS_TRANSIENT => UnicastAddressFlags::Transient,
            0 => UnicastAddressFlags::NoFlags,
            o => UnicastAddressFlags::Other(o),
        }
    }
}

struct AdapterAddresses(Vec<u8>);

impl AdapterAddresses {
    pub fn new() -> Result<AdapterAddresses, WinError> {
        // Start with a 16kb buffer
        let mut adapters = vec![0u8; 16384];
        let mut size = adapters.len() as u32;

        match WIN32_ERROR(unsafe {
            GetAdaptersAddresses(
                AF_UNSPEC.0.into(),
                adapter_flags(),
                None,
                Some(adapters.as_mut_ptr().cast()),
                &mut size,
            )
        })
        .ok()
        {
            Ok(_) => (),
            Err(e) if e == ERROR_BUFFER_OVERFLOW.into() => {
                println!("New size: {:x}", size);
                adapters.resize(size as usize, 0);

                WIN32_ERROR(unsafe {
                    GetAdaptersAddresses(
                        AF_UNSPEC.0.into(),
                        adapter_flags(),
                        None,
                        Some(adapters.as_mut_ptr().cast()),
                        &mut size,
                    )
                })
                .ok()?
            }
            Err(e) => return Err(WinError::from(e)),
        };

        adapters.truncate(size as usize);

        Ok(AdapterAddresses(adapters))
    }

    pub fn entries(&mut self) -> AdapterAddressEntries {
        AdapterAddressEntries {
            adapters: self.0.as_mut_ptr().cast(),
            _marker: PhantomData,
        }
    }
}

#[repr(transparent)]
struct AdapterAddressEntries<'a> {
    adapters: *mut IP_ADAPTER_ADDRESSES_LH,
    _marker: PhantomData<&'a IP_ADAPTER_ADDRESSES_LH>,
}

impl<'a> Iterator for AdapterAddressEntries<'a> {
    type Item = AdapterAddressEntry<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let entry = AdapterAddressEntry {
            adapter: NonNull::new(self.adapters)?,
            _marker: PhantomData,
        };

        // No need to NULL check since self.adapters since it is checked above
        self.adapters = unsafe { (*self.adapters).Next };

        Some(entry)
    }
}

#[repr(transparent)]
struct AdapterAddressEntry<'a> {
    adapter: NonNull<IP_ADAPTER_ADDRESSES_LH>,
    _marker: PhantomData<&'a IP_ADAPTER_ADDRESSES_LH>,
}

impl<'a> AdapterAddressEntry<'_> {
    pub fn unicast_addresses(&self) -> UnicastAddresses {
        UnicastAddresses {
            addresses: unsafe { self.adapter.as_ref() }.FirstUnicastAddress,
            _marker: PhantomData,
        }
    }

    pub fn operstatus(&self) -> IfOperStatus {
        unsafe { self.adapter.as_ref() }.OperStatus.into()
    }
}

#[repr(transparent)]
struct UnicastAddresses<'a> {
    addresses: *mut IP_ADAPTER_UNICAST_ADDRESS_LH,
    _marker: PhantomData<&'a IP_ADAPTER_UNICAST_ADDRESS_LH>,
}

impl<'a> Iterator for UnicastAddresses<'a> {
    type Item = UnicastAddress<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let entry = UnicastAddress {
            address: NonNull::new(self.addresses)?,
            _marker: PhantomData,
        };

        self.addresses = unsafe { (*self.addresses).Next };
        Some(entry)
    }
}

#[repr(transparent)]
struct UnicastAddress<'a> {
    address: NonNull<IP_ADAPTER_UNICAST_ADDRESS_LH>,
    _marker: PhantomData<&'a IP_ADAPTER_UNICAST_ADDRESS_LH>,
}

impl<'a> UnicastAddress<'_> {
    pub fn address(&self) -> Option<IpvSocketAddress> {
        let socketaddr = unsafe { self.address.as_ref() }.Address;

        if unsafe { (*socketaddr.lpSockaddr).sa_family } == AF_INET {
            Some(IpvSocketAddress::Ipv4(Ipv4SocketAddress {
                addr: NonNull::new(socketaddr.lpSockaddr.cast())?,
                _marker: PhantomData,
            }))
        } else if unsafe { (*socketaddr.lpSockaddr).sa_family } == AF_INET6 {
            Some(IpvSocketAddress::Ipv6(Ipv6SocketAddress {
                addr: NonNull::new(socketaddr.lpSockaddr.cast())?,
                _marker: PhantomData,
            }))
        } else {
            None
        }
    }

    pub fn flags(&self) -> UnicastAddressFlags {
        unsafe { self.address.as_ref().Anonymous.Anonymous }
            .Flags
            .into()
    }
}

enum IpvSocketAddress<'a> {
    Ipv4(Ipv4SocketAddress<'a>),
    Ipv6(Ipv6SocketAddress<'a>),
}

impl<'a> IpvSocketAddress<'_> {
    pub fn to_string_opt(&self) -> Option<String> {
        match self {
            IpvSocketAddress::Ipv4(a) => a.to_string_opt(),
            IpvSocketAddress::Ipv6(a) => a.to_string_opt(),
        }
    }
}

#[repr(transparent)]
struct Ipv4SocketAddress<'a> {
    addr: NonNull<SOCKADDR_IN>,
    _marker: PhantomData<&'a SOCKADDR_IN>,
}

impl<'a> Ipv4SocketAddress<'_> {
    pub fn to_string_opt(&self) -> Option<String> {
        let mut buffer = [0u8; 16];
        if unsafe {
            inet_ntop(
                AF_INET.0.into(),
                std::ptr::addr_of!(self.addr.as_ref().sin_addr).cast(),
                buffer.as_mut_slice(),
            )
        }
        .is_null()
        {
            return None;
        }

        Some(
            std::ffi::CStr::from_bytes_until_nul(&buffer)
                .ok()?
                .to_string_lossy()
                .to_string(),
        )
    }
}

#[repr(transparent)]
struct Ipv6SocketAddress<'a> {
    addr: NonNull<SOCKADDR_IN6>,
    _marker: PhantomData<&'a SOCKADDR_IN6>,
}

impl<'a> Ipv6SocketAddress<'_> {
    pub fn to_string_opt(&self) -> Option<String> {
        let mut buffer = [0u8; 46];
        if unsafe {
            inet_ntop(
                AF_INET6.0.into(),
                std::ptr::addr_of!(self.addr.as_ref().sin6_addr).cast(),
                buffer.as_mut_slice(),
            )
        }
        .is_null()
        {
            return None;
        }

        Some(
            std::ffi::CStr::from_bytes_until_nul(&buffer)
                .ok()?
                .to_string_lossy()
                .to_string(),
        )
    }
}

pub fn internal_ips() -> Result<Vec<String>, WinError> {
    let mut adapters = AdapterAddresses::new()?;

    Ok(adapters
        .entries()
        .filter(|entry| entry.operstatus() == IfOperStatus::Up)
        .flat_map(|adapter| {
            adapter
                .unicast_addresses()
                .filter(|unicast_address| {
                    unicast_address.flags() == UnicastAddressFlags::DnsEligible
                })
                .flat_map(|unicast_address| {
                    unicast_address
                        .address()
                        .map(|address| address.to_string_opt())
                        .flatten()
                })
                .collect::<Vec<String>>()
        })
        .collect())
}

#[cfg(test)]
mod tests {
    #[test]
    fn internal_ip_test() {
        let ip_addresses = super::internal_ips().unwrap();
        println!("{:?}", ip_addresses);
    }
}
