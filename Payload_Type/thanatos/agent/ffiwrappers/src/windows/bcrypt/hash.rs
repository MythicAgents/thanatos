use std::marker::PhantomData;

use generic_array::GenericArray;
use windows::Win32::Security::Cryptography::{
    BCryptDestroyHash, BCryptFinishHash, BCryptHashData, BCRYPT_HASH_HANDLE,
};

use super::{traits::HashAlgorithm, BCryptAlgHandle};

#[repr(transparent)]
pub struct BCryptHashHandle<T: HashAlgorithm> {
    pub(super) handle: BCRYPT_HASH_HANDLE,
    pub(super) _marker: PhantomData<BCRYPT_HASH_HANDLE>,
    pub(super) _ty: PhantomData<T>,
}

impl<T: HashAlgorithm> BCryptHashHandle<T> {
    pub fn new() -> BCryptHashHandle<T> {
        let mut alg_handle = BCryptAlgHandle::<T>::new();
        alg_handle.create_hash()
    }

    pub fn hash_data(&mut self, data: &[u8]) {
        // Possible return/error values are documented here: https://learn.microsoft.com/en-us/windows/win32/api/bcrypt/nf-bcrypt-bcrypthashdata#return-value
        // Error assertions:
        // - STATUS_INVALID_PARAMETER:
        //   - The flags parameter is hard coded as 0 for no flags. Since this is hard coded
        //     as 0, an invalid parameter error should never be returned.
        // - STATUS_INVALID_HANDLE:
        //   - This error is returned if the hash handle is invalid. This method is only
        //     callable if a valid hash handle is created. This object can only be
        //     created from a valid algorithm handle. There will never be an instance
        //     where this is called with an invalid handle.
        //
        // SAFETY: Error assertions are defined above.
        let _ = unsafe { BCryptHashData(self.handle, data, 0) };
    }

    pub fn finish_hash(self) -> GenericArray<u8, T::LEN> {
        let mut output: GenericArray<u8, T::LEN> = Default::default();

        // Possible return/error values are documented here: https://learn.microsoft.com/en-us/windows/win32/api/bcrypt/nf-bcrypt-bcryptfinishhash
        // Error assertions:
        // - STATUS_INVALID_HANDLE:
        //   - This is only reachable if a valid hash handle is created. The handle will
        //     always be valid.
        // - STATUS_INVALID_PARAMETER:
        //   - The flags parameter is hard coded as 0 for no flags. The length of the output
        //     array is checked at compile time. The array size will always be the correct
        //     length. The hash size is also compile time checked. An invalid parameter
        //     error code will never be returned.
        //
        // SAFETY: Error assertions are defined above.
        let _ = unsafe { BCryptFinishHash(self.handle, output.as_mut_slice(), 0) };

        output
    }
}

impl<T: HashAlgorithm> Default for BCryptHashHandle<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: HashAlgorithm> Drop for BCryptHashHandle<T> {
    fn drop(&mut self) {
        let _ = unsafe { BCryptDestroyHash(self.handle) };
    }
}

#[cfg(test)]
mod tests {
    use crate::windows::bcrypt::{algorithms::Sha256, BCryptAlgHandle};

    use super::BCryptHashHandle;

    const WORD: &'static str = "hello";

    const EXPECTED: [u8; 32] =
        hex_literal::hex!("2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824");

    #[test]
    fn sha256_alg_test() {
        let mut alg = BCryptAlgHandle::<Sha256>::new();
        let mut h = alg.create_hash();
        h.hash_data(WORD.as_bytes());

        let output: [u8; 32] = h.finish_hash().into();
        assert_eq!(output, EXPECTED);
    }

    #[test]
    fn sha256_new_test() {
        let mut h = BCryptHashHandle::<Sha256>::new();
        h.hash_data(WORD.as_bytes());
        let output: [u8; 32] = h.finish_hash().into();
        assert_eq!(output, EXPECTED);
    }
}
