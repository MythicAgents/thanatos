use crate::crypto::windows::traits::BCryptRandomAlgorithm;

use windows::{
    core::Error as WinError,
    Win32::Security::Cryptography::{BCryptGenRandom, BCRYPT_USE_SYSTEM_PREFERRED_RNG},
};

use super::BCryptAlgorithmHandle;

impl<A: BCryptRandomAlgorithm> BCryptAlgorithmHandle<A> {
    pub fn fill_bytes(&self, bytes: &mut [u8]) -> Result<(), WinError> {
        unsafe { BCryptGenRandom(self.handle, bytes, BCRYPT_USE_SYSTEM_PREFERRED_RNG).ok()? }
        Ok(())
    }
}
