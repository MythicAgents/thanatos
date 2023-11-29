//! Helper functions for doing AES/RSA crypto

#[cfg(feature = "cryptinternal")]
pub mod internal;

#[cfg(feature = "cryptinternal")]
pub use internal::*;

#[cfg(not(feature = "cryptinternal"))]
mod system;

#[cfg(not(feature = "cryptinternal"))]
pub use system::*;

use crate::errors::ThanatosError;

/// Required trait for objects which implement RSA routines
pub trait RsaImpl {
    /// Generates a new RSA key pair with the specified bits
    fn generate(bits: usize) -> Result<Self, ThanatosError>
    where
        Self: Sized;

    /// Returns the public RSA key in PEM format
    fn public_key_pem(&self) -> Result<String, ThanatosError>;

    /// Decrypts a message with the RSA private key
    fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>, ThanatosError>;
}
