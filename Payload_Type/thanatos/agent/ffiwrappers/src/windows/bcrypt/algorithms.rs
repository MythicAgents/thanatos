use super::{Algorithm, HashAlgorithm};
use generic_array::typenum::U32;
use windows::{core::PCWSTR, Win32::Security::Cryptography::BCRYPT_SHA256_ALGORITHM};

pub struct Sha256;

impl super::internal::Private for Sha256 {}

impl HashAlgorithm for Sha256 {
    type LEN = U32;
}

impl Algorithm for Sha256 {
    const ALGID: PCWSTR = BCRYPT_SHA256_ALGORITHM;
}
