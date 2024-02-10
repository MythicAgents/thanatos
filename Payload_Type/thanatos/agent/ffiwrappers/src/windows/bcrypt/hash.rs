use std::marker::PhantomData;

use errors::ThanatosError;
use generic_array::{sequence::GenericSequence, GenericArray};
use windows::Win32::Security::Cryptography::{
    BCryptDestroyHash, BCryptFinishHash, BCryptHashData, BCRYPT_HASH_HANDLE,
};

use super::traits::HashAlgorithm;

#[repr(transparent)]
pub struct BCryptHashHandle<T: HashAlgorithm> {
    pub(super) handle: BCRYPT_HASH_HANDLE,
    pub(super) _marker: PhantomData<BCRYPT_HASH_HANDLE>,
    pub(super) _ty: PhantomData<T>,
}

impl<T: HashAlgorithm> BCryptHashHandle<T> {
    pub fn hash_data(&mut self, data: &[u8]) -> Result<(), ThanatosError> {
        unsafe { BCryptHashData(self.handle, data, 0) }
            .ok()
            .map_err(ThanatosError::from_windows)
    }

    pub fn finish_hash(self) -> Result<GenericArray<u8, T::LEN>, ThanatosError> {
        let mut output = GenericArray::<u8, T::LEN>::generate(|v| v as u8);

        unsafe { BCryptFinishHash(self.handle, output.as_mut_slice(), 0) }
            .ok()
            .map_err(ThanatosError::from_windows)?;

        Ok(output)
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

    #[test]
    fn sha256_test() {
        let w = "hello";

        let expected =
            hex_literal::hex!("2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824");

        let mut alg = BCryptAlgHandle::<Sha256>::new(None).unwrap();
        let mut h = alg.create_hash().unwrap();
        h.hash_data(w.as_bytes()).unwrap();

        let output: [u8; 32] = h.finish_hash().unwrap().into();
        assert_eq!(output, expected);
    }
}
