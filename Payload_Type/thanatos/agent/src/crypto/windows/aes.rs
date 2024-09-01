use std::marker::PhantomData;

use windows::{
    core::Error as WinError,
    Win32::Security::Cryptography::{
        BCryptGenerateSymmetricKey, BCRYPT_AES_ALGORITHM, BCRYPT_KEY_HANDLE,
    },
};

use crate::crypto::errors::AesError;

use super::{
    bcrypt::{BCryptAlgorithmHandle, BCryptKeyHandle},
    pkcs7_pad, pkcs7_unpad,
    traits::{
        private, BCryptAlgorithm, BCryptAlgorithmIV, BCryptKeyAlgorithm, BCryptSymmetricAlgorithm,
    },
};

struct AesAlgorithm;
impl private::Sealed for AesAlgorithm {}
impl BCryptAlgorithm for AesAlgorithm {
    const ALGID: windows::core::PCWSTR = BCRYPT_AES_ALGORITHM;
}

impl BCryptKeyAlgorithm for AesAlgorithm {}
impl BCryptSymmetricAlgorithm for AesAlgorithm {}
impl BCryptAlgorithmIV for AesAlgorithm {}

impl<A: BCryptSymmetricAlgorithm> BCryptAlgorithmHandle<A> {
    pub fn import_symmetric_key(&self, key: &[u8]) -> Result<BCryptKeyHandle<A>, WinError> {
        let mut key_handle = BCRYPT_KEY_HANDLE::default();
        unsafe { BCryptGenerateSymmetricKey(self.handle, &mut key_handle, None, key, 0) }.ok()?;

        Ok(BCryptKeyHandle {
            handle: key_handle,
            _algorithm: PhantomData,
        })
    }
}

/// AES256 encrypts data in place without copying. Data needs to be padded!
pub fn encrypt_aes256(mut iv: [u8; 16], key: &[u8], data: Vec<u8>) -> Result<Vec<u8>, AesError> {
    let key_handle = BCryptAlgorithmHandle::<AesAlgorithm>::new()
        .map_err(|e| AesError::WinError(e.code()))?
        .import_symmetric_key(key)
        .map_err(|e| AesError::WinError(e.code()))?;

    let mut data = pkcs7_pad(data, 16);

    key_handle
        .encrypt_iv(&mut iv, &mut data)
        .map_err(|e| AesError::WinError(e.code()))?;

    Ok(data)
}

/// AES256 decrypts the data in place without copying
pub fn decrypt_aes256(
    mut iv: [u8; 16],
    key: &[u8],
    mut data: Vec<u8>,
) -> Result<Vec<u8>, AesError> {
    let key_handle = BCryptAlgorithmHandle::<AesAlgorithm>::new()
        .map_err(|e| AesError::WinError(e.code()))?
        .import_symmetric_key(key)
        .map_err(|e| AesError::WinError(e.code()))?;

    key_handle
        .decrypt_iv(&mut iv, &mut data)
        .map_err(|e| AesError::WinError(e.code()))?;

    Ok(pkcs7_unpad(data))
}
