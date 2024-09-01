use generic_array::typenum::U32;
use windows::Win32::Security::Cryptography::BCRYPT_SHA256_ALGORITHM;

use crate::crypto::errors::Sha256Error;

use super::{
    bcrypt::BCryptHashHandle,
    traits::{private, BCryptAlgorithm, BCryptHashAlgorithm},
};

pub(super) struct Sha256Algorithm;
impl private::Sealed for Sha256Algorithm {}
impl BCryptAlgorithm for Sha256Algorithm {
    const ALGID: windows::core::PCWSTR = BCRYPT_SHA256_ALGORITHM;
}

impl BCryptHashAlgorithm for Sha256Algorithm {
    type HashLen = U32;
}

#[repr(transparent)]
#[allow(dead_code)]
pub struct Sha256(BCryptHashHandle<Sha256Algorithm>);

impl Sha256 {
    #[allow(dead_code)]
    pub fn new() -> Result<Sha256, Sha256Error> {
        Ok(Self(
            BCryptHashHandle::new().map_err(|e| Sha256Error::WinError(e.code()))?,
        ))
    }

    #[allow(dead_code)]
    pub fn update(&mut self, data: &[u8]) -> Result<(), Sha256Error> {
        self.0
            .update(data)
            .map_err(|e| Sha256Error::WinError(e.code()))
    }

    #[allow(dead_code)]
    pub fn finalize(self) -> Result<[u8; 32], Sha256Error> {
        self.0
            .finalize()
            .map_err(|e| Sha256Error::WinError(e.code()))
            .map(|h| h.into())
    }
}
