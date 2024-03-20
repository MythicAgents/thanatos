pub mod crypto_rng;
pub mod errors;
pub mod xoshiro;

#[cfg(windows)]
mod windows;

#[cfg(windows)]
pub use windows::{aes, base64, hmac, rng, rsa};

use self::{
    aes::{decrypt_aes256, encrypt_aes256},
    errors::{CryptoError, HmacError},
    hmac::HmacSha256,
};

/// Encrypt and sign a message
pub fn encrypt_message(key: &[u8], message: Vec<u8>) -> Result<Vec<u8>, CryptoError> {
    let mut iv = [0u8; 16];
    crypto_rng::thread_rng().fill_bytes(&mut iv)?;

    let mut output = Vec::from(iv.clone());

    let mut message = encrypt_aes256(iv, key, message).map_err(CryptoError::Aes)?;
    output.append(&mut message);

    let mut h = HmacSha256::new(key).map_err(CryptoError::Hmac)?;
    h.update(&output).map_err(CryptoError::Hmac)?;
    output.extend(h.finalize().map_err(CryptoError::Hmac)?);

    Ok(output)
}

pub fn decrypt_message(key: &[u8], mut message: Vec<u8>) -> Result<Vec<u8>, CryptoError> {
    let hmac: [u8; 32] = message[message.len() - 32..]
        .try_into()
        .map_err(|_| CryptoError::InvalidData)?;

    let mut h = HmacSha256::new(key).map_err(CryptoError::Hmac)?;
    h.update(&message[..message.len() - 32])
        .map_err(CryptoError::Hmac)?;

    let calculated_hmac = h.finalize().map_err(CryptoError::Hmac)?;
    if calculated_hmac != hmac {
        return Err(CryptoError::Hmac(HmacError::MacMismatch));
    }

    let iv: [u8; 16] = message[..16]
        .try_into()
        .map_err(|_| CryptoError::InvalidData)?;

    message.drain(..16);
    message.drain(message.len() - 32..);
    decrypt_aes256(iv, key, message).map_err(CryptoError::Aes)
}
