use std::error::Error;

use super::bcrypt::{BCryptAlgorithm, BCryptAlgorithmHandle};

pub fn encrypt(key: &[u8], iv: &[u8], data: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
    let algorithm_handle = BCryptAlgorithmHandle::new(BCryptAlgorithm::Aes)?;
    let key_handle = algorithm_handle.generate_symmetric_key(key)?;
    key_handle.encrypt(iv, data)
}

pub fn calc_sha256_hmac(key: &[u8], data: &[u8]) -> Result<[u8; 32], Box<dyn Error>> {
    let algorithm_handle = BCryptAlgorithmHandle::new_hmac(BCryptAlgorithm::Sha256)?;
    let hmac = algorithm_handle.hash_data(data, Some(key))?;
    Ok(<[u8; 32]>::try_from(hmac).unwrap())
}

pub fn decrypt(key: &[u8], iv: &[u8], data: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
    let algorithm_handle = BCryptAlgorithmHandle::new(BCryptAlgorithm::Aes)?;
    let key_handle = algorithm_handle.generate_symmetric_key(key)?;
    key_handle.decrypt(iv, data)
}
