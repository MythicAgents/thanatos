use std::marker::PhantomData;

use windows::core::Error as WinError;
use windows::Win32::Security::Cryptography::{
    BCryptCloseAlgorithmProvider, BCryptDestroyHash, BCryptDestroyKey, BCryptOpenAlgorithmProvider,
    BCRYPT_ALG_HANDLE, BCRYPT_HASH_HANDLE, BCRYPT_KEY_HANDLE, BCRYPT_OPEN_ALGORITHM_PROVIDER_FLAGS,
};

use super::traits::{BCryptAlgorithm, BCryptHashAlgorithm, BCryptKeyAlgorithm};

pub mod asymmetric;
pub mod hash;
pub mod hmac;
pub mod symmetric;
pub mod random;

#[repr(transparent)]
pub struct BCryptAlgorithmHandle<A: BCryptAlgorithm> {
    pub handle: BCRYPT_ALG_HANDLE,
    _algorithm: PhantomData<A>,
}

impl<A: BCryptAlgorithm> BCryptAlgorithmHandle<A> {
    #[inline(always)]
    pub fn new() -> Result<BCryptAlgorithmHandle<A>, WinError> {
        Self::new_opt(0)
    }

    #[inline(always)]
    fn new_opt(opt: u32) -> Result<BCryptAlgorithmHandle<A>, WinError> {
        Self::new_flags(BCRYPT_OPEN_ALGORITHM_PROVIDER_FLAGS(opt))
    }

    #[inline(always)]
    fn new_flags(
        flags: BCRYPT_OPEN_ALGORITHM_PROVIDER_FLAGS,
    ) -> Result<BCryptAlgorithmHandle<A>, WinError> {
        let mut handle = BCRYPT_ALG_HANDLE::default();
        unsafe { BCryptOpenAlgorithmProvider(&mut handle, A::ALGID, None, flags) }.ok()?;

        Ok(Self {
            handle,
            _algorithm: PhantomData,
        })
    }

    #[inline(always)]
    pub fn as_inner(&self) -> BCRYPT_ALG_HANDLE {
        self.handle
    }
}

impl<A: BCryptAlgorithm> Drop for BCryptAlgorithmHandle<A> {
    fn drop(&mut self) {
        let _ = unsafe { BCryptCloseAlgorithmProvider(self.handle, 0) };
    }
}

#[repr(transparent)]
pub struct BCryptKeyHandle<A: BCryptKeyAlgorithm> {
    pub handle: BCRYPT_KEY_HANDLE,
    pub _algorithm: PhantomData<A>,
}

impl<A: BCryptKeyAlgorithm> Drop for BCryptKeyHandle<A> {
    fn drop(&mut self) {
        let _ = unsafe { BCryptDestroyKey(self.handle) };
    }
}

#[repr(transparent)]
pub struct BCryptHashHandle<A: BCryptHashAlgorithm> {
    handle: BCRYPT_HASH_HANDLE,
    _algorithm: PhantomData<A>,
}

impl<A: BCryptHashAlgorithm> Drop for BCryptHashHandle<A> {
    fn drop(&mut self) {
        let _ = unsafe { BCryptDestroyHash(self.handle) };
    }
}
