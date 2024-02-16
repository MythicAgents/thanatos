use std::marker::PhantomData;

use windows::{
    core::Error as WinError,
    Win32::{
        Foundation::{STATUS_BUFFER_TOO_SMALL, STATUS_NO_MEMORY},
        Security::Cryptography::{
            BCryptCloseAlgorithmProvider, BCryptCreateHash, BCryptOpenAlgorithmProvider,
            BCRYPT_ALG_HANDLE, BCRYPT_HASH_HANDLE, BCRYPT_OPEN_ALGORITHM_PROVIDER_FLAGS,
            MS_PRIMITIVE_PROVIDER,
        },
    },
};

use super::{
    BCryptHashHandle, {Algorithm, HashAlgorithm},
};

#[repr(transparent)]
pub struct BCryptAlgHandle<T: Algorithm> {
    handle: BCRYPT_ALG_HANDLE,
    _marker: PhantomData<BCRYPT_ALG_HANDLE>,
    _ty: PhantomData<T>,
}

impl<T: Algorithm> BCryptAlgHandle<T> {
    pub fn new() -> BCryptAlgHandle<T> {
        let mut handle = BCRYPT_ALG_HANDLE::default();

        // Possible return/error values are documented here: https://learn.microsoft.com/en-us/windows/win32/api/bcrypt/nf-bcrypt-bcryptopenalgorithmprovider#return-value
        // Error assertions:
        // - STATUS_NOT_FOUND:
        //   - Algorithms are strictly type checked using the `Algorithm` trait.
        //     Only algorithms manually defined in this crate are able to be used.
        //     This means that the `BCryptOpenAlgorithmProvider` will never return
        //     a `STATUS_NOT_FOUND` error unless an invalid algorithm is defined in
        //     this crate.
        // - STATUS_INVALID_PARAMETER:
        //   - All of the parameters to this call are hard coded except for the algorithm ID.
        //     The algorithm ID is strictly enforced (outlined above). This error
        //     value will never be returned.
        // - STATUS_NO_MEMORY:
        //   - Only return error value which cannot be eliminated at compile time.
        //     Since a failed memory allocation is a rare occurance and leaves the entire
        //     program in an unstable state, just panic and end execution.
        //
        // Check for a `STATUS_NO_MEMORY` error and panic if the system is out of memory.
        //
        // SAFETY: Parameter values are hard coded and strictly checked. Errors need
        // to be handled but only to check for `STATUS_NO_MEMORY` errors. Refer
        // to the error assertions above.
        match unsafe {
            BCryptOpenAlgorithmProvider(
                &mut handle,
                T::ALGID,
                MS_PRIMITIVE_PROVIDER,
                BCRYPT_OPEN_ALGORITHM_PROVIDER_FLAGS(0),
            )
        }
        .ok()
        {
            Ok(_) => (),
            Err(e) if e.code() == WinError::from(STATUS_NO_MEMORY).code() => {
                panic!("OOM");
            }
            _ => unreachable!(),
        }

        BCryptAlgHandle {
            handle,
            _marker: PhantomData,
            _ty: PhantomData,
        }
    }
}

impl<T: Algorithm> Default for BCryptAlgHandle<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: HashAlgorithm> BCryptAlgHandle<T> {
    pub fn create_hash(&mut self) -> BCryptHashHandle<T> {
        let mut hash_handle = BCRYPT_HASH_HANDLE::default();

        // Possible return/error values are documented here: https://learn.microsoft.com/en-us/windows/win32/api/bcrypt/nf-bcrypt-bcryptcreatehash#return-value
        // Error assertions:
        // - STATUS_BUFFER_TOO_SMALL:
        //   - The `cbHashObject` parameter is optional. Specifying NULL or in out case `None`
        //     for the `pbhashobject` parameter means that Windows is handling
        //     the memory for the hash object. This error code should never be returned
        //     because Windows is handling the hash object.
        // - STATUS_INVALID_HANDLE:
        //   - The algorithm handle is coming from this algorithm object. This algorithm
        //     object can only be created with a valid algorithm ID. This method is also
        //     only reachable when a valid algorithm handle is created. Thus, this error
        //     value will never be returned.
        // - STATUS_INVALID_PARAMETER:
        //   - The only variable values are the algorithm handle an the pointer to the
        //     resulting hash handle. The rest of the parameters are hard coded.
        //     The flags variable is also set to define no flags. There is never
        //     an instance where this receives an invalid parameter.
        // - STATUS_NOT_SUPPORTED:
        //   - This error is returned if the algorithm ID for the algorithm handle does not support
        //     hashing. Only algorithms which implement the `HashAlgorithm` trait are able to
        //     call this function. There is never an instance where this function is
        //     called with an algorithm that does not implement the hash trait.
        //
        // Check for a `STATUS_BUFFER_TOO_SMALL` error just in case Windows runs
        // into an error managing memory. If this is the case, panic since it's unrecoverable.
        //
        // SAFETY: Parameter values are hard coded and strictly checked. Errors need
        // to be handled but only to check for `STATUS_NO_MEMORY` errors. Refer
        // to the error assertions above.
        match unsafe { BCryptCreateHash(self.handle, &mut hash_handle, None, None, 0) }.ok() {
            Ok(_) => (),
            Err(e) if e.code() == WinError::from(STATUS_BUFFER_TOO_SMALL).code() => {
                panic!("OOM");
            }
            _ => unreachable!(),
        }

        BCryptHashHandle {
            handle: hash_handle,
            _marker: PhantomData,
            _ty: PhantomData,
        }
    }
}

impl<T: Algorithm> Drop for BCryptAlgHandle<T> {
    fn drop(&mut self) {
        let _ = unsafe { BCryptCloseAlgorithmProvider(self.handle, 0) };
    }
}

#[cfg(test)]
mod tests {
    use crate::windows::bcrypt::algorithms::Sha256;

    use super::*;

    #[test]
    fn bcrypt_alg_test() {
        let _ = BCryptAlgHandle::<Sha256>::new();
    }
}
