use std::marker::PhantomData;

use generic_array::GenericArray;

use windows::core::Error as WinError;
use windows::Win32::Security::Cryptography::{
    BCryptCreateHash, BCryptFinishHash, BCryptHashData, BCRYPT_ALG_HANDLE_HMAC_FLAG,
    BCRYPT_HASH_HANDLE,
};

use crate::crypto::windows::traits::BCryptHashAlgorithm;

use super::{BCryptAlgorithmHandle, BCryptHashHandle};

impl<A: BCryptHashAlgorithm> BCryptAlgorithmHandle<A> {
    pub fn hmac(key: &[u8]) -> Result<BCryptHashHandle<A>, WinError> {
        let algorithm_handle = Self::new_flags(BCRYPT_ALG_HANDLE_HMAC_FLAG)?;

        let mut hash_handle = BCRYPT_HASH_HANDLE::default();
        unsafe {
            BCryptCreateHash(
                algorithm_handle.handle,
                &mut hash_handle,
                None,
                Some(key),
                0,
            )
        }
        .ok()?;

        Ok(BCryptHashHandle {
            handle: hash_handle,
            _algorithm: PhantomData,
        })
    }
}

impl<A: BCryptHashAlgorithm> BCryptHashHandle<A> {
    #[allow(dead_code)]
    pub fn new() -> Result<BCryptHashHandle<A>, WinError> {
        let algorithm_handle = BCryptAlgorithmHandle::<A>::new()?;

        let mut hash_handle = BCRYPT_HASH_HANDLE::default();
        unsafe { BCryptCreateHash(algorithm_handle.handle, &mut hash_handle, None, None, 0) }
            .ok()?;

        Ok(BCryptHashHandle {
            handle: hash_handle,
            _algorithm: PhantomData,
        })
    }

    pub fn hmac(key: &[u8]) -> Result<BCryptHashHandle<A>, WinError> {
        BCryptAlgorithmHandle::<A>::hmac(key)
    }

    pub fn update(&mut self, data: &[u8]) -> Result<(), WinError> {
        unsafe { BCryptHashData(self.handle, data, 0).ok() }
    }

    pub fn finalize(self) -> Result<GenericArray<u8, A::HashLen>, WinError> {
        let mut buffer: GenericArray<u8, A::HashLen> = GenericArray::default();
        unsafe { BCryptFinishHash(self.handle, buffer.as_mut_slice(), 0) }.ok()?;
        Ok(buffer)
    }
}
