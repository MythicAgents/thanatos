//! Helpers for interacting with the Windows API. The majority of this is RAII wrappers.

use crate::ThanatosError;
use std::marker::PhantomData;

use windows::Win32::{
    Foundation::{ERROR_BUFFER_OVERFLOW, ERROR_SUCCESS},
    NetworkManagement::IpHelper::{GetAdaptersInfo, IP_ADAPTER_INFO},
};

/// RAII wrapper for the Windows interfaces linked list
pub struct IpAdapters<'a> {
    /// Buffer containing the IP Adapters linked list
    _adapters_buffer: Vec<u8>,

    /// Current element in the IP adapters linked list for iterating
    adapters_curr: *const IP_ADAPTER_INFO,

    /// Marker needed for returning a reference in the iterator
    _marker: PhantomData<&'a IP_ADAPTER_INFO>,
}

impl IpAdapters<'_> {
    /// Get the IP adapters
    pub fn new() -> Result<Self, ThanatosError> {
        let mut adapters_len = 0u32;

        let ret = unsafe { GetAdaptersInfo(None, &mut adapters_len) };
        if ret != ERROR_BUFFER_OVERFLOW.0 {
            return Err(ThanatosError::from_error_code(ret as i32));
        }

        let mut adapters_buffer = vec![0u8; adapters_len as usize];

        let ret = unsafe {
            GetAdaptersInfo(Some(adapters_buffer.as_mut_ptr().cast()), &mut adapters_len)
        };

        if ret != ERROR_SUCCESS.0 {
            return Err(ThanatosError::from_error_code(ret as i32));
        }

        let adapters_curr = adapters_buffer.as_ptr().cast();

        Ok(Self {
            adapters_curr,
            _adapters_buffer: adapters_buffer,
            _marker: PhantomData,
        })
    }
}

impl<'a> Iterator for IpAdapters<'a> {
    type Item = &'a IP_ADAPTER_INFO;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.adapters_curr.is_null() {
            let adapter_copy = self.adapters_curr;
            self.adapters_curr = unsafe { *adapter_copy }.Next;

            Some(unsafe { &*adapter_copy })
        } else {
            None
        }
    }
}
