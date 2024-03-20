use std::marker::PhantomData;

use windows::Win32::Security::Cryptography::{
    BCryptFinalizeKeyPair, BCryptGenerateKeyPair, BCRYPT_RSA_ALGORITHM,
};
use windows::{core::Error as WinError, Win32::Security::Cryptography::BCRYPT_KEY_HANDLE};

use crate::crypto::errors::{CryptoError, RsaError};

use super::traits::{private, BCryptAlgorithm, BCryptAsymmetricAlgorithm, BCryptKeyAlgorithm};
use super::{
    base64,
    bcrypt::{BCryptAlgorithmHandle, BCryptKeyHandle},
};

struct RsaAlgorithm;
impl private::Sealed for RsaAlgorithm {}
impl BCryptAlgorithm for RsaAlgorithm {
    const ALGID: windows::core::PCWSTR = BCRYPT_RSA_ALGORITHM;
}

impl BCryptKeyAlgorithm for RsaAlgorithm {}
impl BCryptAsymmetricAlgorithm for RsaAlgorithm {}

impl BCryptKeyHandle<RsaAlgorithm> {
    fn generate(bits: u32) -> Result<BCryptKeyHandle<RsaAlgorithm>, WinError> {
        let algorithm_handle = BCryptAlgorithmHandle::<RsaAlgorithm>::new()?;

        let mut key_handle = BCRYPT_KEY_HANDLE::default();
        unsafe { BCryptGenerateKeyPair(algorithm_handle.as_inner(), &mut key_handle, bits, 0) }
            .ok()?;

        unsafe { BCryptFinalizeKeyPair(key_handle, 0) }.ok()?;

        Ok(BCryptKeyHandle {
            handle: key_handle,
            _algorithm: PhantomData,
        })
    }
}

#[repr(transparent)]
pub struct Rsa(BCryptKeyHandle<RsaAlgorithm>);

impl Rsa {
    pub fn generate(bits: u32) -> Result<Rsa, CryptoError> {
        Ok(Self(BCryptKeyHandle::generate(bits).map_err(|e| {
            CryptoError::Rsa(RsaError::WinError(e.code()))
        })?))
    }

    pub fn public_key(&self) -> Result<String, CryptoError> {
        let pub_key_asn = self
            .0
            .export_public_key()
            .map_err(|e| CryptoError::Rsa(RsaError::WinError(e.code())))?;

        let pub_key = base64::encode(pub_key_asn);

        Ok(format!(
            "{}\n{}{}",
            "-----BEGIN RSA PUBLIC KEY-----", pub_key, "-----END RSA PUBLIC KEY-----"
        ))
    }

    pub fn private_decrypt(&self, data: &[u8]) -> Result<Vec<u8>, CryptoError> {
        self.0
            .private_decrypt(data)
            .map_err(|e| CryptoError::Rsa(RsaError::WinError(e.code())))
    }
}

#[cfg(test)]
mod tests {
    use super::Rsa;

    #[test]
    fn debug_rsa() {
        let r = Rsa::generate(4096).unwrap();
        let pub_key = r.public_key().unwrap();
        dbg!(pub_key);
    }
}
