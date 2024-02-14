mod alghandle;
pub use alghandle::BCryptAlgHandle;

mod hash;
pub use hash::BCryptHashHandle;

pub mod algorithms;

pub mod traits {
    use generic_array::ArrayLength;
    use windows::core::PCWSTR;

    pub trait Algorithm {
        const ALGID: PCWSTR;
    }

    pub trait HashAlgorithm: Algorithm {
        type LEN: ArrayLength;
    }
}
