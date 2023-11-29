//! Crypto routines when doing internal crypto

use crate::ThanatosError;
use aes::cipher::{BlockEncryptMut, KeyIvInit};
use base64::{engine::general_purpose, Engine as _};
use block_padding::Pkcs7;
use hmac::{Hmac, Mac};
use rand::RngCore;
use rsa::{
    pkcs1::{EncodeRsaPublicKey, LineEnding},
    Oaep, RsaPrivateKey, RsaPublicKey,
};
use sha2::{Digest, Sha256};

type Aes256CbcEnc = cbc::Encryptor<aes::Aes256>;
type HmacSha256 = Hmac<Sha256>;

/// RSA key pair for internal crypto
pub struct RsaKeyPair {
    private_key: RsaPrivateKey,
    public_key: RsaPublicKey,
}

impl super::RsaImpl for RsaKeyPair {
    fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>, ThanatosError> {
        let padding = Oaep::new::<sha1::Sha1>();
        self.private_key
            .decrypt(padding, data)
            .map_err(|_| ThanatosError::RsaDecryptError)
    }

    fn public_key_pem(&self) -> Result<String, ThanatosError> {
        self.public_key
            .to_pkcs1_pem(LineEnding::LF)
            .map_err(|_| ThanatosError::RsaKeyGenerateError)
    }

    fn generate(bits: usize) -> Result<Self, ThanatosError> {
        let mut rng = rand::rngs::OsRng;

        let private_key =
            RsaPrivateKey::new(&mut rng, bits).map_err(|_| ThanatosError::RsaKeyGenerateError)?;
        let public_key = RsaPublicKey::from(&private_key);

        Ok(Self {
            private_key,
            public_key,
        })
    }
}

/// Wrapper to base64 encode data.
pub fn b64encode(data: impl AsRef<[u8]>) -> String {
    general_purpose::STANDARD.encode(data)
}

/// Wrapper to base64 decode data.
pub fn b64decode(data: impl AsRef<str>) -> Result<Vec<u8>, ThanatosError> {
    general_purpose::STANDARD
        .decode(data.as_ref().as_bytes())
        .map_err(|_| ThanatosError::Base64DecodeError)
}

/// AES256 encrypts data with a supplied key
pub fn encrypt_aes(key: &[u8; 32], data: impl AsRef<[u8]>) -> Result<Vec<u8>, ThanatosError> {
    let mut iv = [0u8; 16];
    rand::rngs::OsRng.fill_bytes(&mut iv);

    let cipher = Aes256CbcEnc::new(key.into(), &iv.into());
    let mut ciphertext = cipher.encrypt_padded_vec_mut::<Pkcs7>(data.as_ref());

    let mut msg = Vec::from(iv);
    msg.append(&mut ciphertext);

    let mut mac = HmacSha256::new_from_slice(key).map_err(|_| ThanatosError::AesEncryptError)?;
    mac.update(&msg);

    let mac = mac.finalize();
    msg.extend(&mac.into_bytes());

    Ok(msg)
}

/// AES256 decrypts data with the supplied key and verifies the hmac
#[cfg(any(feature = "AES", feature = "EKE"))]
pub fn decrypt_aes_verify(
    key: &[u8; 32],
    data: impl AsRef<[u8]>,
) -> Result<Vec<u8>, ThanatosError> {
    use crate::debug_invoke;
    use aes::cipher::BlockDecryptMut;

    type Aes256CbcDec = cbc::Decryptor<aes::Aes256>;

    let iv: [u8; 16] =
        <[u8; 16]>::try_from(&data.as_ref()[..16]).map_err(|_| ThanatosError::AesDecryptError)?;

    let signed_data = &data.as_ref()[..data.as_ref().len() - 32];

    let encrypted_data = &data.as_ref()[16..data.as_ref().len() - 32];

    let signature = &data.as_ref()[data.as_ref().len() - 32..];
    let signature = debug_invoke!(
        <[u8; 32]>::try_from(signature),
        ThanatosError::AesDecryptError
    );

    let mut mac = HmacSha256::new_from_slice(key).map_err(|_| ThanatosError::AesDecryptError)?;
    mac.update(&signed_data);

    debug_invoke!(
        mac.verify(&signature.into()),
        ThanatosError::MessageSignatureMismatch
    );

    let cipher = Aes256CbcDec::new(key.into(), &iv.into());
    let plaintext = cipher
        .decrypt_padded_vec_mut::<Pkcs7>(encrypted_data)
        .map_err(|_| ThanatosError::AesDecryptError)?;

    Ok(plaintext)
}

/// Gets the sha256 hash of some data
pub fn sha256(data: impl AsRef<[u8]>) -> Result<[u8; 32], ThanatosError> {
    let mut hasher = Sha256::new();
    hasher.update(data);
    Ok(hasher.finalize().into())
}
