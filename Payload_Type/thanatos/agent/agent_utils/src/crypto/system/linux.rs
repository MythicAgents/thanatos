//! Crypto routines when doing encryption using system libraries on Linux. This uses openssl
//! for all of the encryption schemes.

use crate::ThanatosError;
use openssl::{
    cipher::Cipher,
    cipher_ctx::CipherCtx,
    hash::MessageDigest,
    pkey::PKey,
    pkey::Private,
    rsa::{Padding, Rsa},
    sha,
    sign::Signer,
};

use rand::RngCore;

/// RSA key pair for Linux system crypto using openssl
pub struct RsaKeyPair {
    private_key: Rsa<Private>,
}

impl crate::crypto::RsaImpl for RsaKeyPair {
    fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>, ThanatosError> {
        let mut decrypted_data = vec![0u8; data.len()];
        self.private_key
            .private_decrypt(data, &mut decrypted_data, Padding::PKCS1_OAEP)
            .map_err(|_| ThanatosError::RsaDecryptError)?;

        Ok(decrypted_data)
    }

    fn public_key_pem(&self) -> Result<String, ThanatosError> {
        std::str::from_utf8(
            &self
                .private_key
                .public_key_to_pem_pkcs1()
                .map_err(|_| ThanatosError::RsaKeyGenerateError)?,
        )
        .map_err(|_| ThanatosError::StringParseError)
        .map(|s| s.to_string())
    }

    fn generate(bits: usize) -> Result<Self, ThanatosError> {
        let private_key =
            Rsa::generate(bits as u32).map_err(|_| ThanatosError::RsaKeyGenerateError)?;

        Ok(Self { private_key })
    }
}

/// Wrapper to base64 encode data.
pub fn b64encode(data: impl AsRef<[u8]>) -> String {
    openssl::base64::encode_block(data.as_ref())
}

/// Wrapper to base64 decode data.
pub fn b64decode(data: impl AsRef<str>) -> Result<Vec<u8>, ThanatosError> {
    openssl::base64::decode_block(data.as_ref()).map_err(|_| ThanatosError::Base64DecodeError)
}

/// AES256 encrypts data with a supplied key using openssl
pub fn encrypt_aes(key: &[u8; 32], data: impl AsRef<[u8]>) -> Result<Vec<u8>, ThanatosError> {
    let mut iv = [0u8; 16];
    rand::rngs::OsRng.fill_bytes(&mut iv);

    let cipher = Cipher::aes_256_cbc();

    let mut ctx = CipherCtx::new().map_err(|_| ThanatosError::AesEncryptError)?;
    ctx.encrypt_init(Some(cipher), Some(key), Some(&iv))
        .map_err(|_| ThanatosError::AesEncryptError)?;

    let mut ciphertext = Vec::new();
    ctx.cipher_update_vec(data.as_ref(), &mut ciphertext)
        .map_err(|_| ThanatosError::AesEncryptError)?;
    ctx.cipher_final_vec(&mut ciphertext)
        .map_err(|_| ThanatosError::AesEncryptError)?;

    let mut msg = Vec::from(iv);
    msg.append(&mut ciphertext);

    let mac_key = PKey::hmac(key.as_slice()).map_err(|_| ThanatosError::AesEncryptError)?;

    let mut signer = Signer::new(MessageDigest::sha256(), &mac_key)
        .map_err(|_| ThanatosError::AesEncryptError)?;
    signer
        .update(&msg)
        .map_err(|_| ThanatosError::AesEncryptError)?;

    let hmac = signer
        .sign_to_vec()
        .map_err(|_| ThanatosError::AesEncryptError)?;
    msg.extend(&hmac[..32]);

    Ok(msg)
}

/// AES256 decrypts data with the supplied key and verifies the hmac using openssl
#[cfg(any(feature = "AES", feature = "EKE"))]
pub fn decrypt_aes_verify(
    key: &[u8; 32],
    data: impl AsRef<[u8]>,
) -> Result<Vec<u8>, ThanatosError> {
    let iv: [u8; 16] =
        <[u8; 16]>::try_from(&data.as_ref()[..16]).map_err(|_| ThanatosError::AesDecryptError)?;

    let signed_data = &data.as_ref()[..data.as_ref().len() - 32];
    let encrypted_data = &data.as_ref()[16..data.as_ref().len() - 32];
    let signature = <[u8; 32]>::try_from(&data.as_ref()[data.as_ref().len() - 32..])
        .map_err(|_| ThanatosError::AesDecryptError)?;

    let mac_key = PKey::hmac(key.as_slice()).map_err(|_| ThanatosError::AesDecryptError)?;
    let mut signer = Signer::new(MessageDigest::sha256(), &mac_key)
        .map_err(|_| ThanatosError::AesDecryptError)?;

    signer
        .update(&signed_data)
        .map_err(|_| ThanatosError::AesDecryptError)?;

    let hmac = signer
        .sign_to_vec()
        .map_err(|_| ThanatosError::AesDecryptError)?;

    if !openssl::memcmp::eq(&hmac, &signature) {
        return Err(ThanatosError::MessageSignatureMismatch);
    }

    let cipher = Cipher::aes_256_cbc();

    let mut ctx = CipherCtx::new().map_err(|_| ThanatosError::AesEncryptError)?;
    ctx.decrypt_init(Some(cipher), Some(key), Some(&iv))
        .map_err(|_| ThanatosError::AesEncryptError)?;

    let mut plaintext = Vec::new();
    ctx.cipher_update_vec(encrypted_data, &mut plaintext)
        .map_err(|_| ThanatosError::AesDecryptError)?;
    ctx.cipher_final_vec(&mut plaintext)
        .map_err(|_| ThanatosError::AesDecryptError)?;

    Ok(plaintext)
}

/// Gets the sha256 hash of some data
pub fn sha256(data: impl AsRef<[u8]>) -> Result<[u8; 32], ThanatosError> {
    let mut hasher = sha::Sha256::new();
    hasher.update(data.as_ref());
    Ok(hasher.finish())
}
