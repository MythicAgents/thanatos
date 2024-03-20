use windows::{
    core::Error as WinError,
    Win32::{
        Foundation::STATUS_INVALID_BUFFER_SIZE,
        Security::Cryptography::{BCryptDecrypt, BCryptEncrypt, BCRYPT_FLAGS},
    },
};

use crate::crypto::windows::traits::{BCryptAlgorithmIV, BCryptSymmetricAlgorithm};

use super::BCryptKeyHandle;

impl<A: BCryptSymmetricAlgorithm + BCryptAlgorithmIV> BCryptKeyHandle<A> {
    /// Encrypts the data in place
    pub fn encrypt_iv(&self, iv: &mut [u8], data: &mut [u8]) -> Result<(), WinError> {
        // There is a very small chance that this can overflow if the data being
        // sent back is larger than 4gb. This should NEVER happen since the agent
        // will chunk the messages and never send back more than a reasonable
        // amount of data at a time.
        let mut data_len = data
            .len()
            .try_into()
            .map_err(|_| WinError::from_hresult(STATUS_INVALID_BUFFER_SIZE.to_hresult()))?;

        // Create an immutable reference to the data for the input. Need to break
        // Rust's borrowing semantics because there needs to be both a mutable
        // and immutable reference to the data being encrypted since the encryption
        // is done in place
        let input: &[u8] = unsafe { std::slice::from_raw_parts(data.as_ptr(), data.len()) };

        // Encrypt the data in place.
        // The remarks section for `BCryptEncrypt` state that the input and output
        // can be equal as long as the buffer is large enough to hold all the
        // encrypted data. Padding is added above so the output buffer should be valid.
        //
        // https://learn.microsoft.com/en-us/windows/win32/api/bcrypt/nf-bcrypt-bcryptencrypt#remarks
        // > The pbInput and pbOutput parameters can be equal. In this case, this function will perform the encryption in place.
        // > It is possible that the encrypted data size will be larger than the unencrypted data size, so the buffer must be large enough to hold the encrypted data.
        // > If pbInput and pbOutput are not equal then the two buffers may not overlap.
        unsafe {
            BCryptEncrypt(
                self.handle,
                Some(input),
                None,
                Some(iv),
                Some(data),
                &mut data_len,
                BCRYPT_FLAGS(0),
            )
            .ok()?
        };

        Ok(())
    }

    pub fn decrypt_iv(&self, iv: &mut [u8], data: &mut [u8]) -> Result<(), WinError> {
        // Same as above. This could possibly overflow in the case that the
        // agent receives back more than 4gb of data from Mythic.
        let mut data_len = data
            .len()
            .try_into()
            .map_err(|_| WinError::from_hresult(STATUS_INVALID_BUFFER_SIZE.to_hresult()))?;

        let input: &[u8] = unsafe { std::slice::from_raw_parts(data.as_ptr(), data.len()) };

        // Decrypt the data in place.
        // The remarks section for `BCryptDecrypt` state that the input and output
        // buffers can be equal.
        //
        // https://learn.microsoft.com/en-us/windows/win32/api/bcrypt/nf-bcrypt-bcryptdecrypt#remark
        // > The pbInput and pbOutput parameters can be equal.
        // > In this case, this function will perform the decryption in place.
        // > If pbInput and pbOutput are not equal, the two buffers may not overlap
        unsafe {
            BCryptDecrypt(
                self.handle,
                Some(input),
                None,
                Some(iv),
                Some(data),
                &mut data_len,
                BCRYPT_FLAGS(0),
            )
            .ok()?
        };

        Ok(())
    }
}
