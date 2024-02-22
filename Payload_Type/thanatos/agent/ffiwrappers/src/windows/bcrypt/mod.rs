use generic_array::ArrayLength;
use windows::core::PCWSTR;

mod alghandle;
pub use alghandle::BCryptAlgHandle;

mod hash;
pub use hash::BCryptHashHandle;

pub mod algorithms;

pub trait Algorithm: crate::internal::SealedTrait {
    const ALG: PCWSTR;
}

// So until generic constant expressions are stabilized, unfortunately have to use `generic_array`.
// Each hash algorithm length is a type instead of a constant. This is rather annoying
// since it requires using the numeric type mappings from typenum.
// Please Rust stabilize generic const exprs, thanks :)
pub trait HashAlgorithm: Algorithm + crate::internal::SealedTrait {
    type LEN: ArrayLength;
}
