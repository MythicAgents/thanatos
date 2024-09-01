use windows::core::Error as WinError;
use windows::core::PCSTR;
use windows::Win32::Security::Cryptography::{
    BCryptDecrypt, CryptExportPublicKeyInfoFromBCryptKeyHandle, BCRYPT_OAEP_PADDING_INFO,
    BCRYPT_PAD_OAEP, BCRYPT_SHA1_ALGORITHM, CERT_PUBLIC_KEY_INFO,
    CRYPT_OID_INFO_PUBKEY_ENCRYPT_KEY_FLAG, X509_ASN_ENCODING,
};

use crate::crypto::windows::traits::BCryptAsymmetricAlgorithm;

use super::BCryptKeyHandle;

impl<A: BCryptAsymmetricAlgorithm> BCryptKeyHandle<A> {
    pub fn export_public_key(&self) -> Result<Vec<u8>, WinError> {
        let mut blob_size = 0u32;
        unsafe {
            CryptExportPublicKeyInfoFromBCryptKeyHandle(
                self.handle,
                X509_ASN_ENCODING,
                PCSTR::null(),
                CRYPT_OID_INFO_PUBKEY_ENCRYPT_KEY_FLAG.0,
                None,
                None,
                &mut blob_size,
            )
            .ok()?
        };

        let mut blob_buffer = vec![0u8; blob_size as usize];
        unsafe {
            CryptExportPublicKeyInfoFromBCryptKeyHandle(
                self.handle,
                X509_ASN_ENCODING,
                PCSTR::null(),
                CRYPT_OID_INFO_PUBKEY_ENCRYPT_KEY_FLAG.0,
                None,
                Some(blob_buffer.as_mut_ptr().cast()),
                &mut blob_size,
            )
            .ok()?
        };

        let blob_buffer = blob_buffer;
        let blob_ptr = blob_buffer.as_ptr() as *const CERT_PUBLIC_KEY_INFO;

        Ok(unsafe {
            Vec::from_raw_parts(
                (*blob_ptr).PublicKey.pbData,
                (*blob_ptr).PublicKey.cbData as usize,
                (*blob_ptr).PublicKey.cbData as usize,
            )
        })
    }

    pub fn private_decrypt(&self, data: &[u8]) -> Result<Vec<u8>, WinError> {
        let padding_info = BCRYPT_OAEP_PADDING_INFO {
            pszAlgId: BCRYPT_SHA1_ALGORITHM,
            pbLabel: std::ptr::null_mut(),
            cbLabel: 0,
        };

        let mut decrypted_length = 0u32;
        unsafe {
            BCryptDecrypt(
                self.handle,
                Some(data),
                Some(std::ptr::addr_of!(padding_info).cast()),
                None,
                None,
                &mut decrypted_length,
                BCRYPT_PAD_OAEP,
            )
            .ok()?
        }

        let mut decrypted_buffer = vec![0u8; decrypted_length as usize];
        unsafe {
            BCryptDecrypt(
                self.handle,
                Some(data),
                Some(std::ptr::addr_of!(padding_info).cast()),
                None,
                Some(&mut decrypted_buffer),
                &mut decrypted_length,
                BCRYPT_PAD_OAEP,
            )
            .ok()?
        }

        Ok(decrypted_buffer)
    }
}
