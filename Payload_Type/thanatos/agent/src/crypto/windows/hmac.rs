use crate::crypto::errors::HmacError;

use super::{bcrypt::BCryptHashHandle, sha::Sha256Algorithm};

#[repr(transparent)]
pub struct HmacSha256(BCryptHashHandle<Sha256Algorithm>);

impl HmacSha256 {
    pub fn new(key: &[u8]) -> Result<HmacSha256, HmacError> {
        Ok(HmacSha256(
            BCryptHashHandle::<Sha256Algorithm>::hmac(key)
                .map_err(|e| HmacError::WinError(e.code()))?,
        ))
    }

    pub fn update(&mut self, data: &[u8]) -> Result<(), HmacError> {
        self.0
            .update(data)
            .map_err(|e| HmacError::WinError(e.code()))
    }

    pub fn finalize(self) -> Result<[u8; 32], HmacError> {
        self.0
            .finalize()
            .map_err(|e| HmacError::WinError(e.code()))
            .map(|h| h.into())
    }
}
