use super::{Algorithm, HashAlgorithm};
use generic_array::typenum::U32;
use windows::{core::PCWSTR, Win32::Security::Cryptography::BCRYPT_SHA256_ALGORITHM};

pub struct Sha256;

impl crate::internal::SealedTrait for Sha256 {}

impl HashAlgorithm for Sha256 {
    type LEN = U32;
}

impl Algorithm for Sha256 {
    const ALG: PCWSTR = BCRYPT_SHA256_ALGORITHM;
}
