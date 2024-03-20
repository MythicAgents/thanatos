use windows::Win32::Security::Cryptography::BCRYPT_RNG_ALGORITHM;

use crate::crypto::errors::{CryptoError, RngError};

use super::{
    bcrypt::BCryptAlgorithmHandle,
    traits::{private, BCryptAlgorithm, BCryptRandomAlgorithm},
};

struct RngAlgorithm;
impl private::Sealed for RngAlgorithm {}
impl BCryptAlgorithm for RngAlgorithm {
    const ALGID: windows::core::PCWSTR = BCRYPT_RNG_ALGORITHM;
}

impl BCryptRandomAlgorithm for RngAlgorithm {}

#[repr(transparent)]
pub struct Rng(BCryptAlgorithmHandle<RngAlgorithm>);

impl Rng {
    pub fn new() -> Result<Rng, CryptoError> {
        Ok(Self(BCryptAlgorithmHandle::<RngAlgorithm>::new().map_err(
            |e| CryptoError::Rng(RngError::WinError(e.code())),
        )?))
    }

    pub fn fill_bytes(&self, bytes: &mut [u8]) -> Result<(), CryptoError> {
        self.0
            .fill_bytes(bytes)
            .map_err(|e| CryptoError::Rng(RngError::WinError(e.code())))
    }
}
