use std::{error::Error, ffi::c_void};

use windows::{
    core::{PCSTR, PCWSTR},
    Win32::Security::Cryptography::{
        BCryptCloseAlgorithmProvider, BCryptDecrypt, BCryptDestroyKey, BCryptEncrypt,
        BCryptFinalizeKeyPair, BCryptGenerateKeyPair, BCryptGenerateSymmetricKey,
        BCryptGetProperty, BCryptHash, BCryptOpenAlgorithmProvider,
        CryptExportPublicKeyInfoFromBCryptKeyHandle, BCRYPT_AES_ALGORITHM, BCRYPT_ALG_HANDLE,
        BCRYPT_ALG_HANDLE_HMAC_FLAG, BCRYPT_FLAGS, BCRYPT_HASH_LENGTH, BCRYPT_KEY_HANDLE,
        BCRYPT_OPEN_ALGORITHM_PROVIDER_FLAGS, BCRYPT_RSA_ALGORITHM, BCRYPT_SHA256_ALGORITHM,
        CERT_QUERY_ENCODING_TYPE,
    },
};

pub enum BCryptAlgorithm {
    Rsa,
    Aes,
    Sha256,
}

impl From<BCryptAlgorithm> for PCWSTR {
    fn from(value: BCryptAlgorithm) -> Self {
        match value {
            BCryptAlgorithm::Rsa => BCRYPT_RSA_ALGORITHM,
            BCryptAlgorithm::Aes => BCRYPT_AES_ALGORITHM,
            BCryptAlgorithm::Sha256 => BCRYPT_SHA256_ALGORITHM,
        }
    }
}

fn pkcs7_pad(data: &[u8], blocksize: usize) -> Vec<u8> {
    let mut padded = Vec::with_capacity(data.len().next_multiple_of(blocksize));
    let v = (blocksize - (data.len() % blocksize)) as u8;
    padded.extend_from_slice(data);
    padded.extend_from_slice(&(0..v as usize).map(|_| v).collect::<Vec<u8>>());
    padded
}

fn pkcs7_unpad(data: &[u8], blocksize: usize) -> Vec<u8> {
    let l = data[data.len() - 1];
    if l as usize > blocksize {
        return data.to_vec();
    }

    let p = &data[data.len() - l as usize..];
    if p.iter().all(|v| v == &l) {
        data[..data.len() - l as usize].to_vec()
    } else {
        data.to_vec()
    }
}

#[repr(transparent)]
pub struct BCryptAlgorithmHandle(BCRYPT_ALG_HANDLE);

impl BCryptAlgorithmHandle {
    pub fn new(algid: BCryptAlgorithm) -> Result<BCryptAlgorithmHandle, Box<dyn Error>> {
        Self::new_opt(algid, 0)
    }

    pub fn new_hmac(algid: BCryptAlgorithm) -> Result<BCryptAlgorithmHandle, Box<dyn Error>> {
        Self::new_opt(algid, BCRYPT_ALG_HANDLE_HMAC_FLAG.0)
    }

    fn new_opt(
        algid: BCryptAlgorithm,
        flags: u32,
    ) -> Result<BCryptAlgorithmHandle, Box<dyn Error>> {
        let mut handle = BCRYPT_ALG_HANDLE::default();
        unsafe {
            BCryptOpenAlgorithmProvider(
                &mut handle,
                Into::<PCWSTR>::into(algid),
                None,
                BCRYPT_OPEN_ALGORITHM_PROVIDER_FLAGS(flags),
            )
            .ok()?
        };

        Ok(Self(handle))
    }

    pub fn generate_keypair(&self, bits: u32) -> Result<BCryptAsymmetricKeyHandle, Box<dyn Error>> {
        let mut key_handle = BCRYPT_KEY_HANDLE::default();
        unsafe { BCryptGenerateKeyPair(self.0, &mut key_handle, bits, 0).ok()? };
        unsafe { BCryptFinalizeKeyPair(key_handle, 0).ok()? };
        Ok(BCryptAsymmetricKeyHandle(key_handle))
    }

    pub fn generate_symmetric_key(
        &self,
        key: &[u8],
    ) -> Result<BCryptSymmetricKeyHandle, Box<dyn Error>> {
        let mut key_handle = BCRYPT_KEY_HANDLE::default();
        unsafe { BCryptGenerateSymmetricKey(self.0, &mut key_handle, None, key, 0).ok()? };
        Ok(BCryptSymmetricKeyHandle(key_handle))
    }

    pub fn hash_data(&self, data: &[u8], secret: Option<&[u8]>) -> Result<Vec<u8>, Box<dyn Error>> {
        let mut hash_len_buffer = [0u8; 4];
        let mut hash_len_buffer_len = hash_len_buffer.len() as u32;

        unsafe {
            BCryptGetProperty(
                self.0,
                BCRYPT_HASH_LENGTH,
                Some(&mut hash_len_buffer),
                &mut hash_len_buffer_len,
                0,
            )
            .ok()?
        };

        let hash_len = u32::from_le_bytes(hash_len_buffer);
        let mut hash_buffer = vec![0u8; hash_len as usize];

        unsafe {
            BCryptHash(self.0, secret, data, &mut hash_buffer)
                .ok()
                .unwrap()
        };
        Ok(hash_buffer)
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

pub struct BCryptSymmetricKeyHandle(BCRYPT_KEY_HANDLE);

impl BCryptSymmetricKeyHandle {
    pub fn encrypt(&self, iv: &[u8], data: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
        let mut ciphertext_len = 0u32;
        let mut iv_copy = Vec::from(iv);

        let padded_data = pkcs7_pad(data, 16);

        unsafe {
            BCryptEncrypt(
                self.0,
                Some(&padded_data),
                None,
                Some(&mut iv_copy),
                None,
                &mut ciphertext_len,
                BCRYPT_FLAGS(0),
            )
            .ok()?
        };

        let mut iv_copy = Vec::from(iv);
        let mut ciphertext = vec![0u8; ciphertext_len as usize];

        unsafe {
            BCryptEncrypt(
                self.0,
                Some(&padded_data),
                None,
                Some(&mut iv_copy),
                Some(&mut ciphertext),
                &mut ciphertext_len,
                BCRYPT_FLAGS(0),
            )
            .ok()?
        };

        Ok(ciphertext)
    }

    pub fn decrypt(&self, iv: &[u8], data: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
        let mut plaintext_len = 0u32;
        let mut iv_copy = Vec::from(iv);

        unsafe {
            BCryptDecrypt(
                self.0,
                Some(data),
                None,
                Some(&mut iv_copy),
                None,
                &mut plaintext_len,
                BCRYPT_FLAGS(0),
            )
            .ok()?
        };

        let mut iv_copy = Vec::from(iv);
        let mut plaintext = vec![0u8; plaintext_len as usize];

        unsafe {
            BCryptDecrypt(
                self.0,
                Some(data),
                None,
                Some(&mut iv_copy),
                Some(&mut plaintext),
                &mut plaintext_len,
                BCRYPT_FLAGS(0),
            )
            .ok()?
        };

        Ok(pkcs7_unpad(&plaintext, 16))
    }
}

impl Drop for BCryptSymmetricKeyHandle {
    fn drop(&mut self) {
        unsafe {
            BCryptDestroyKey(self.0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{BCryptAlgorithm, BCryptAlgorithmHandle};

    #[test]
    fn hash_property() {
        let alg_handle = BCryptAlgorithmHandle::new(BCryptAlgorithm::Sha256).unwrap();
        let d = [0u8; 32];
        let s = [0u8; 32];
        alg_handle.hash_data(&d, Some(&s)).unwrap();
    }
}
