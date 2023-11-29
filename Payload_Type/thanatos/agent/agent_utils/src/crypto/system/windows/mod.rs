//! Crypto routines when doing encryption using system libraries on Windows.
//! This uses [Windows CNG](https://learn.microsoft.com/en-us/windows/win32/seccng/cng-reference)
//! for all the encryption schemes.

mod crypto_handles;

use rand::RngCore;
use std::os::windows::prelude::AsRawHandle;

use crate::ThanatosError;
use windows::{
    core::{PCWSTR, PSTR},
    Win32::Security::Cryptography::{
        BCryptCreateHash, BCryptDecrypt, BCryptEncrypt, BCryptFinishHash, BCryptGenerateKeyPair,
        BCryptGenerateSymmetricKey, BCryptGetProperty, BCryptHashData, BCryptOpenAlgorithmProvider,
        CryptBinaryToStringA, CryptStringToBinaryA, BCRYPT_AES_ALGORITHM, BCRYPT_ALG_HANDLE,
        BCRYPT_ALG_HANDLE_HMAC_FLAG, BCRYPT_BLOCK_PADDING, BCRYPT_HANDLE, BCRYPT_HASH_HANDLE,
        BCRYPT_HASH_LENGTH, BCRYPT_KEY_HANDLE, BCRYPT_OBJECT_LENGTH,
        BCRYPT_OPEN_ALGORITHM_PROVIDER_FLAGS, BCRYPT_RSA_ALGORITHM, BCRYPT_SHA256_ALGORITHM,
        CRYPT_STRING, CRYPT_STRING_BASE64, CRYPT_STRING_NOCRLF,
    },
};

#[cfg(debug_assertions)]
use windows::Win32::Foundation::GetLastError;

/// RSA key pair for Windows system crypto using Windows CNG
pub struct RsaKeyPair {
    handle: crypto_handles::OwnedBCryptKeyHandle,
}

impl crate::crypto::RsaImpl for RsaKeyPair {
    fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>, ThanatosError> {
        todo!();
    }

    fn public_key_pem(&self) -> Result<String, ThanatosError> {
        todo!();
    }

    fn generate(bits: usize) -> Result<Self, ThanatosError> {
        let mut alg_handle = BCRYPT_ALG_HANDLE(-1isize);
        crate::debug_invoke!(
            unsafe {
                BCryptOpenAlgorithmProvider(
                    &mut alg_handle,
                    BCRYPT_RSA_ALGORITHM,
                    PCWSTR(std::ptr::null()),
                    BCRYPT_OPEN_ALGORITHM_PROVIDER_FLAGS(0),
                )
            },
            ThanatosError::AesEncryptError
        );

        let alg_handle = crypto_handles::OwnedAlgorithmProviderHandle::try_from(alg_handle.0)?;

        let mut key_handle = BCRYPT_KEY_HANDLE(-1isize);

        crate::debug_invoke!(
            unsafe {
                BCryptGenerateKeyPair(
                    BCRYPT_ALG_HANDLE(alg_handle.as_handle().as_raw_handle() as _),
                    &mut key_handle,
                    bits as u32,
                    0,
                )
            },
            ThanatosError::AesEncryptError,
            { crate::log!("{:?}", unsafe { GetLastError() }) }
        );

        let key_handle = crypto_handles::OwnedBCryptKeyHandle::try_from(key_handle.0)?;

        Ok(Self { handle: key_handle })
    }
}

/// Wrapper to base64 encode data.
pub fn b64encode(data: impl AsRef<[u8]>) -> String {
    let mut encoded_size = 0u32;
    unsafe {
        CryptBinaryToStringA(
            data.as_ref(),
            CRYPT_STRING(CRYPT_STRING_BASE64.0 | CRYPT_STRING_NOCRLF),
            PSTR(std::ptr::null_mut()),
            &mut encoded_size,
        )
    };

    let mut encoded_data = vec![0u8; encoded_size as usize];

    unsafe {
        CryptBinaryToStringA(
            data.as_ref(),
            CRYPT_STRING(CRYPT_STRING_BASE64.0 | CRYPT_STRING_NOCRLF),
            PSTR(encoded_data.as_mut_ptr().cast()),
            &mut encoded_size,
        )
    };

    encoded_data.pop().unwrap();

    std::str::from_utf8(&encoded_data).unwrap().to_string()
}

/// Wrapper to base64 decode data.
pub fn b64decode(data: impl AsRef<str>) -> Result<Vec<u8>, ThanatosError> {
    let mut decoded_size = 0u32;

    unsafe {
        CryptStringToBinaryA(
            data.as_ref().as_bytes(),
            CRYPT_STRING_BASE64,
            None,
            &mut decoded_size,
            None,
            None,
        )
    };

    let mut decoded_data = vec![0u8; decoded_size as usize];

    crate::debug_invoke!(
        unsafe {
            CryptStringToBinaryA(
                data.as_ref().as_bytes(),
                CRYPT_STRING_BASE64,
                Some(decoded_data.as_mut_ptr().cast()),
                &mut decoded_size,
                None,
                None,
            )
        }
        .ok(),
        ThanatosError::Base64DecodeError,
        { crate::log!("{:?}", unsafe { GetLastError() }) }
    );

    Ok(decoded_data)
}

fn calc_hmac256(key: &[u8; 32], data: &impl AsRef<[u8]>) -> Result<Vec<u8>, ThanatosError> {
    let mut hmac_alg_handle = BCRYPT_ALG_HANDLE(-1isize);
    crate::debug_invoke!(
        unsafe {
            BCryptOpenAlgorithmProvider(
                &mut hmac_alg_handle,
                BCRYPT_SHA256_ALGORITHM,
                PCWSTR(std::ptr::null()),
                BCRYPT_ALG_HANDLE_HMAC_FLAG,
            )
        },
        ThanatosError::CalcHmacError
    );

    let hmac_alg_handle =
        crypto_handles::OwnedAlgorithmProviderHandle::try_from(hmac_alg_handle.0)?;

    let mut hash_object_length = [0u8; std::mem::size_of::<u32>()];
    let mut hash_object_length_size = 0u32;

    crate::debug_invoke!(
        unsafe {
            BCryptGetProperty(
                BCRYPT_HANDLE(hmac_alg_handle.as_handle().as_raw_handle() as _),
                BCRYPT_OBJECT_LENGTH,
                Some(hash_object_length.as_mut_slice()),
                &mut hash_object_length_size,
                0,
            )
        },
        ThanatosError::CalcHmacError
    );

    let hash_object_length = u32::from_le_bytes(hash_object_length);
    let mut hash_object = vec![0u8; hash_object_length as usize];

    let mut hash_length = [0u8; std::mem::size_of::<u32>()];
    let mut hash_length_size = 0u32;

    crate::debug_invoke!(
        unsafe {
            BCryptGetProperty(
                BCRYPT_HANDLE(hmac_alg_handle.as_handle().as_raw_handle() as _),
                BCRYPT_HASH_LENGTH,
                Some(hash_length.as_mut_slice()),
                &mut hash_length_size,
                0,
            )
        },
        ThanatosError::CalcHmacError
    );

    let hash_length = u32::from_le_bytes(hash_length);
    let mut hash_buffer = vec![0u8; hash_length as usize];

    let mut hmac_hash_handle = BCRYPT_HASH_HANDLE(-1isize);

    crate::debug_invoke!(
        unsafe {
            BCryptCreateHash(
                BCRYPT_ALG_HANDLE(hmac_alg_handle.as_handle().as_raw_handle() as _),
                &mut hmac_hash_handle,
                Some(hash_object.as_mut_slice()),
                Some(key.as_slice()),
                0,
            )
        },
        ThanatosError::CalcHmacError
    );

    let hmac_hash_handle = crypto_handles::OwnedBCryptHashHandle::try_from(hmac_hash_handle.0)?;

    crate::debug_invoke!(
        unsafe {
            BCryptHashData(
                BCRYPT_HASH_HANDLE(hmac_hash_handle.as_handle().as_raw_handle() as _),
                data.as_ref(),
                0,
            )
        },
        ThanatosError::CalcHmacError
    );

    crate::debug_invoke!(
        unsafe {
            BCryptFinishHash(
                BCRYPT_HASH_HANDLE(hmac_hash_handle.as_handle().as_raw_handle() as _),
                hash_buffer.as_mut_slice(),
                0,
            )
        },
        ThanatosError::CalcHmacError
    );

    Ok(hash_buffer)
}

/// AES256 encrypts data with a supplied key using Windows CNG
pub fn encrypt_aes(key: &[u8; 32], data: impl AsRef<[u8]>) -> Result<Vec<u8>, ThanatosError> {
    let mut alg_handle = BCRYPT_ALG_HANDLE(-1isize);
    crate::debug_invoke!(
        unsafe {
            BCryptOpenAlgorithmProvider(
                &mut alg_handle,
                BCRYPT_AES_ALGORITHM,
                PCWSTR(std::ptr::null()),
                BCRYPT_OPEN_ALGORITHM_PROVIDER_FLAGS(0),
            )
        },
        ThanatosError::AesEncryptError,
        { crate::log!("{:?}", unsafe { GetLastError() }) }
    );

    let alg_handle = crypto_handles::OwnedAlgorithmProviderHandle::try_from(alg_handle.0)?;

    let mut key_handle = BCRYPT_KEY_HANDLE(0);

    crate::debug_invoke!(
        unsafe {
            BCryptGenerateSymmetricKey(
                BCRYPT_ALG_HANDLE(alg_handle.as_handle().as_raw_handle() as _),
                &mut key_handle,
                None,
                key.as_slice(),
                0,
            )
        },
        ThanatosError::AesEncryptError,
        { crate::log!("{:?}", unsafe { GetLastError() }) }
    );

    let key_handle = crypto_handles::OwnedBCryptKeyHandle::try_from(key_handle.0)?;

    let mut iv = [0u8; 16];
    rand::rngs::OsRng.fill_bytes(&mut iv);

    let mut encrypted_len = 0u32;

    // Need to make a copy of the IV and overwrite it every time `BCryptEncrypt` is called.
    // ref: https://learn.microsoft.com/en-us/windows/win32/api/bcrypt/nf-bcrypt-bcryptencrypt
    // | [in, out, optional] pbIV
    // |
    // | The address of a buffer that contains the initialization vector (IV) to use during encryption.
    // | The cbIV parameter contains the size of this buffer.
    // | "This function will modify the contents of this buffer."
    // | If you need to reuse the IV later, make sure you make a copy of this buffer before calling this function.
    //
    // Honestly....who thought it was a good idea to have this modify the IV?
    let iv_copy = iv.clone();

    let _ = unsafe {
        BCryptEncrypt(
            BCRYPT_KEY_HANDLE(key_handle.as_handle().as_raw_handle() as _),
            Some(data.as_ref()),
            None,
            Some(iv.as_mut_slice()),
            None,
            &mut encrypted_len,
            BCRYPT_BLOCK_PADDING,
        )
    };

    let mut iv = iv_copy;

    let mut ciphertext = vec![0u8; encrypted_len as usize];
    crate::debug_invoke!(
        unsafe {
            BCryptEncrypt(
                BCRYPT_KEY_HANDLE(key_handle.as_handle().as_raw_handle() as _),
                Some(data.as_ref()),
                None,
                Some(iv.as_mut_slice()),
                Some(ciphertext.as_mut_slice()),
                &mut encrypted_len,
                BCRYPT_BLOCK_PADDING,
            )
        },
        ThanatosError::AesEncryptError,
        { crate::log!("{:?}", unsafe { GetLastError() }) }
    );

    let iv = iv_copy;
    let mut msg = Vec::from(iv);
    msg.append(&mut ciphertext);

    let mut hmac_hash = calc_hmac256(key, &msg)?;

    msg.append(&mut hmac_hash);

    Ok(msg)
}

/// AES256 decrypts data with the supplied key and verifies the hmac using Windows CNG
#[cfg(any(feature = "AES", feature = "EKE"))]
pub fn decrypt_aes_verify(
    key: &[u8; 32],
    data: impl AsRef<[u8]>,
) -> Result<Vec<u8>, ThanatosError> {
    let mut iv: [u8; 16] =
        <[u8; 16]>::try_from(&data.as_ref()[..16]).map_err(|_| ThanatosError::AesDecryptError)?;

    let signed_data = &data.as_ref()[..data.as_ref().len() - 32];
    let encrypted_data = &data.as_ref()[16..data.as_ref().len() - 32];

    let signature = &data.as_ref()[data.as_ref().len() - 32..];
    let signature = crate::debug_invoke!(
        <[u8; 32]>::try_from(signature),
        ThanatosError::AesDecryptError
    );

    let hmac_value = calc_hmac256(key, &signed_data)?;

    if hmac_value != signature {
        return Err(ThanatosError::MessageSignatureMismatch);
    }

    let mut alg_handle = BCRYPT_ALG_HANDLE(-1isize);
    crate::debug_invoke!(
        unsafe {
            BCryptOpenAlgorithmProvider(
                &mut alg_handle,
                BCRYPT_AES_ALGORITHM,
                PCWSTR(std::ptr::null()),
                BCRYPT_OPEN_ALGORITHM_PROVIDER_FLAGS(0),
            )
        },
        ThanatosError::AesDecryptError,
        { crate::log!("{:?}", unsafe { GetLastError() }) }
    );

    let alg_handle = crypto_handles::OwnedAlgorithmProviderHandle::try_from(alg_handle.0)?;

    let mut key_handle = BCRYPT_KEY_HANDLE(0);

    crate::debug_invoke!(
        unsafe {
            BCryptGenerateSymmetricKey(
                BCRYPT_ALG_HANDLE(alg_handle.as_handle().as_raw_handle() as _),
                &mut key_handle,
                None,
                key.as_slice(),
                0,
            )
        },
        ThanatosError::AesEncryptError,
        { crate::log!("{:?}", unsafe { GetLastError() }) }
    );

    let key_handle = crypto_handles::OwnedBCryptKeyHandle::try_from(key_handle.0)?;

    let mut decrypted_len = 0u32;

    // Need to make a copy of the IV and overwrite it every time `BCryptDecrypt` is called.
    // ref: https://learn.microsoft.com/en-us/windows/win32/api/bcrypt/nf-bcrypt-bcryptdecrypt
    // | [in, out, optional] pbIV
    // |
    // | The address of a buffer that contains the initialization vector (IV) to use during decryption.
    // | The cbIV parameter contains the size of this buffer.
    // | "This function will modify the contents of this buffer."
    // | If you need to reuse the IV later, make sure you make a copy of this buffer before calling this function.
    // |
    // | This parameter is optional and can be NULL if no IV is used.
    // |
    // | The required size of the IV can be obtained by calling the BCryptGetProperty function to get the BCRYPT_BLOCK_LENGTH property.
    // | This will provide the size of a block for the algorithm, which is also the size of the IV.
    //
    // Okay I'm actually triggered by this.....
    let iv_copy = iv.clone();

    let _ = unsafe {
        BCryptDecrypt(
            BCRYPT_KEY_HANDLE(key_handle.as_handle().as_raw_handle() as _),
            Some(&encrypted_data),
            None,
            Some(iv.as_mut_slice()),
            None,
            &mut decrypted_len,
            BCRYPT_BLOCK_PADDING,
        )
    };

    let mut iv = iv_copy;

    let mut plaintext = vec![0u8; decrypted_len as usize];

    crate::debug_invoke!(
        unsafe {
            BCryptDecrypt(
                BCRYPT_KEY_HANDLE(key_handle.as_handle().as_raw_handle() as _),
                Some(&encrypted_data),
                None,
                Some(iv.as_mut_slice()),
                Some(plaintext.as_mut_slice()),
                &mut decrypted_len,
                BCRYPT_BLOCK_PADDING,
            )
        },
        ThanatosError::AesDecryptError
    );

    plaintext.resize(decrypted_len as usize, 0);
    Ok(plaintext)
}

/// Gets the sha256 hash of some data
pub fn sha256(data: impl AsRef<[u8]>) -> Result<[u8; 32], ThanatosError> {
    let mut hash_alg_handle = BCRYPT_ALG_HANDLE(-1isize);
    crate::debug_invoke!(
        unsafe {
            BCryptOpenAlgorithmProvider(
                &mut hash_alg_handle,
                BCRYPT_SHA256_ALGORITHM,
                PCWSTR(std::ptr::null()),
                BCRYPT_OPEN_ALGORITHM_PROVIDER_FLAGS(0),
            )
        },
        ThanatosError::Sha256HashError
    );

    let hash_alg_handle =
        crypto_handles::OwnedAlgorithmProviderHandle::try_from(hash_alg_handle.0)?;

    let mut hash_handle = BCRYPT_HASH_HANDLE(-1isize);

    crate::debug_invoke!(
        unsafe {
            BCryptCreateHash(
                BCRYPT_ALG_HANDLE(hash_alg_handle.as_handle().as_raw_handle() as _),
                &mut hash_handle,
                None,
                None,
                0,
            )
        },
        ThanatosError::Sha256HashError
    );

    let hash_handle = crypto_handles::OwnedBCryptHashHandle::try_from(hash_handle.0)?;

    crate::debug_invoke!(
        unsafe {
            BCryptHashData(
                BCRYPT_HASH_HANDLE(hash_handle.as_handle().as_raw_handle() as _),
                data.as_ref(),
                0,
            )
        },
        ThanatosError::Sha256HashError
    );

    let mut hash_buffer = [0u8; 32];

    crate::debug_invoke!(
        unsafe {
            BCryptFinishHash(
                BCRYPT_HASH_HANDLE(hash_handle.as_handle().as_raw_handle() as _),
                hash_buffer.as_mut_slice(),
                0,
            )
        },
        ThanatosError::Sha256HashError
    );

    Ok(hash_buffer)
}
