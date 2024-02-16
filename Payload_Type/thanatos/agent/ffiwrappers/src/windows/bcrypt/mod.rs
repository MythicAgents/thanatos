use generic_array::ArrayLength;
use windows::core::PCWSTR;

mod alghandle;
pub use alghandle::BCryptAlgHandle;

mod hash;
pub use hash::BCryptHashHandle;

pub mod algorithms;

mod internal {
    pub trait Private {}
}

pub trait Algorithm: internal::Private {
    const ALGID: PCWSTR;
}

pub trait HashAlgorithm: Algorithm + internal::Private {
    type LEN: ArrayLength;
}
