use std::{error::Error, ffi::c_void};

use windows::{
    core::{PCSTR, PCWSTR},
    Win32::Security::Cryptography::{
        BCryptCloseAlgorithmProvider, BCryptDecrypt, BCryptDestroyKey, BCryptFinalizeKeyPair,
        BCryptGenerateKeyPair, BCryptOpenAlgorithmProvider,
        CryptExportPublicKeyInfoFromBCryptKeyHandle, BCRYPT_ALG_HANDLE, BCRYPT_FLAGS,
        BCRYPT_KEY_HANDLE, BCRYPT_OPEN_ALGORITHM_PROVIDER_FLAGS, BCRYPT_RSA_ALGORITHM,
        CERT_QUERY_ENCODING_TYPE,
    },
};

pub enum BCryptAlgorithm {
    RSA,
}

impl Into<PCWSTR> for BCryptAlgorithm {
    fn into(self) -> PCWSTR {
        match self {
            Self::RSA => BCRYPT_RSA_ALGORITHM,
        }
    }
}

pub enum BCryptHashAlgorithm {}

#[repr(transparent)]
pub struct BCryptAlgorithmHandle(BCRYPT_ALG_HANDLE);

impl BCryptAlgorithmHandle {
    pub fn new(algid: BCryptAlgorithm) -> Result<BCryptAlgorithmHandle, Box<dyn Error>> {
        let mut handle = BCRYPT_ALG_HANDLE::default();
        unsafe {
            BCryptOpenAlgorithmProvider(
                &mut handle,
                Into::<PCWSTR>::into(algid),
                None,
                BCRYPT_OPEN_ALGORITHM_PROVIDER_FLAGS(0),
            )
            .ok()?
        };

        Ok(Self(handle))
    }

    pub fn hmac(algid: BCryptHashAlgorithm) -> Result<BCryptAlgorithmHandle, Box<dyn Error>> {
        todo!();
    }

    pub fn generate_keypair(&self, bits: u32) -> Result<BCryptAsymmetricKeyHandle, Box<dyn Error>> {
        let mut key_handle = BCRYPT_KEY_HANDLE::default();
        unsafe { BCryptGenerateKeyPair(self.0, &mut key_handle, bits, 0).ok()? };
        unsafe { BCryptFinalizeKeyPair(key_handle, 0).ok()? };
        Ok(BCryptAsymmetricKeyHandle(key_handle))
    }
}

impl Drop for BCryptAlgorithmHandle {
    fn drop(&mut self) {
        unsafe {
            BCryptCloseAlgorithmProvider(self.0, 0);
        }
    }
}

#[repr(transparent)]
pub struct BCryptAsymmetricKeyHandle(BCRYPT_KEY_HANDLE);

impl BCryptAsymmetricKeyHandle {
    pub fn export_public_key_info(
        &self,
        encoding: CERT_QUERY_ENCODING_TYPE,
        flags: u32,
    ) -> Result<Vec<u8>, Box<dyn Error>> {
        let mut blob_size = 0u32;
        unsafe {
            CryptExportPublicKeyInfoFromBCryptKeyHandle(
                self.0,
                encoding,
                PCSTR::null(),
                flags,
                None,
                None,
                &mut blob_size,
            )
            .ok()?
        };

        let mut blob_buffer = vec![0u8; blob_size as usize];
        unsafe {
            CryptExportPublicKeyInfoFromBCryptKeyHandle(
                self.0,
                encoding,
                PCSTR::null(),
                flags,
                None,
                Some(blob_buffer.as_mut_ptr().cast()),
                &mut blob_size,
            )
            .ok()?
        };

        Ok(blob_buffer)
    }

    pub fn private_decrypt(
        &self,
        data: &[u8],
        padding_info: Option<*const c_void>,
        flags: BCRYPT_FLAGS,
    ) -> Result<Vec<u8>, Box<dyn Error>> {
        let mut decrypted_length = 0u32;
        unsafe {
            BCryptDecrypt(
                self.0,
                Some(data),
                padding_info,
                None,
                None,
                &mut decrypted_length,
                flags,
            )
            .ok()
        }
        .unwrap();

        let mut decrypted_buffer = vec![0u8; decrypted_length as usize];
        unsafe {
            BCryptDecrypt(
                self.0,
                Some(data),
                padding_info,
                None,
                Some(&mut decrypted_buffer),
                &mut decrypted_length,
                flags,
            )
            .ok()
        }
        .unwrap();

        Ok(decrypted_buffer)
    }
}

impl Drop for BCryptAsymmetricKeyHandle {
    fn drop(&mut self) {
        unsafe {
            BCryptDestroyKey(self.0);
        }
    }
}
