use std::marker::PhantomData;

use errors::ThanatosError;
use windows::{
    core::PCWSTR,
    Win32::Security::Cryptography::{
        BCryptCloseAlgorithmProvider, BCryptCreateHash, BCryptOpenAlgorithmProvider,
        BCRYPT_ALG_HANDLE, BCRYPT_HASH_HANDLE, BCRYPT_OPEN_ALGORITHM_PROVIDER_FLAGS,
        MS_PLATFORM_CRYPTO_PROVIDER, MS_PRIMITIVE_PROVIDER,
    },
};

use super::{
    traits::{Algorithm, HashAlgorithm},
    BCryptHashHandle,
};

pub enum BCryptProvider {
    MsPrimitiveProvider,
    MsPlatformCryptoProvider,
}

impl Into<PCWSTR> for BCryptProvider {
    fn into(self) -> PCWSTR {
        match self {
            BCryptProvider::MsPrimitiveProvider => MS_PRIMITIVE_PROVIDER,
            BCryptProvider::MsPlatformCryptoProvider => MS_PLATFORM_CRYPTO_PROVIDER,
        }
    }
}

#[repr(transparent)]
pub struct BCryptAlgHandle<T: Algorithm> {
    handle: BCRYPT_ALG_HANDLE,
    _marker: PhantomData<BCRYPT_ALG_HANDLE>,
    _ty: PhantomData<T>,
}

impl<T: Algorithm> BCryptAlgHandle<T> {
    pub fn new(
        implementation: Option<BCryptProvider>,
    ) -> Result<BCryptAlgHandle<T>, ThanatosError> {
        let implementation: PCWSTR = implementation.map(|i| i.into()).unwrap_or(PCWSTR::null());
        let mut handle = BCRYPT_ALG_HANDLE::default();

        unsafe {
            BCryptOpenAlgorithmProvider(
                &mut handle,
                T::ALGID,
                implementation,
                BCRYPT_OPEN_ALGORITHM_PROVIDER_FLAGS(0),
            )
        }
        .ok()
        .map_err(ThanatosError::from_windows)?;

        Ok(BCryptAlgHandle {
            handle,
            _marker: PhantomData,
            _ty: PhantomData,
        })
    }
}

impl<T: HashAlgorithm> BCryptAlgHandle<T> {
    pub fn create_hash(&mut self) -> Result<BCryptHashHandle<T>, ThanatosError> {
        let mut hash_handle = BCRYPT_HASH_HANDLE::default();

        unsafe { BCryptCreateHash(self.handle, &mut hash_handle, None, None, 0) }
            .ok()
            .map_err(ThanatosError::from_windows)?;

        Ok(BCryptHashHandle {
            handle: hash_handle,
            _marker: PhantomData,
            _ty: PhantomData,
        })
    }
}

impl<T: Algorithm> Drop for BCryptAlgHandle<T> {
    fn drop(&mut self) {
        let _ = unsafe { BCryptCloseAlgorithmProvider(self.handle, 0) };
    }
}

#[cfg(test)]
mod tests {
    use crate::windows::bcrypt::algorithms::Sha256;

    use super::*;

    #[test]
    fn bcrypt_alg_test() {
        let _ = BCryptAlgHandle::<Sha256>::new(None).unwrap();
    }
}
