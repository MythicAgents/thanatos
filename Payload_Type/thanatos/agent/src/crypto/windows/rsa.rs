use std::error::Error;

use windows::{
    core::PSTR,
    Win32::Security::Cryptography::{
        CryptBinaryToStringA, BCRYPT_OAEP_PADDING_INFO, BCRYPT_PAD_OAEP, BCRYPT_SHA1_ALGORITHM,
        CERT_PUBLIC_KEY_INFO, CRYPT_OID_INFO_PUBKEY_ENCRYPT_KEY_FLAG, CRYPT_STRING,
        CRYPT_STRING_BASE64, CRYPT_STRING_NOCR, X509_ASN_ENCODING,
    },
};

use super::bcrypt::{BCryptAlgorithm, BCryptAlgorithmHandle, BCryptAsymmetricKeyHandle};

pub struct Rsa(BCryptAsymmetricKeyHandle);

impl Rsa {
    pub fn generate(bits: u32) -> Result<Rsa, Box<dyn Error>> {
        let bcrypt_handle = BCryptAlgorithmHandle::new(BCryptAlgorithm::Rsa)?;
        let key_handle = bcrypt_handle.generate_keypair(bits)?;
        Ok(Self(key_handle))
    }

    pub fn public_key(&self) -> Result<String, Box<dyn Error>> {
        let blob_data = self
            .0
            .export_public_key_info(X509_ASN_ENCODING, CRYPT_OID_INFO_PUBKEY_ENCRYPT_KEY_FLAG.0)?;

        let pubkey_blob = blob_data.as_ptr() as *const CERT_PUBLIC_KEY_INFO;

        let pubkey_data = unsafe {
            std::slice::from_raw_parts(
                (*pubkey_blob).PublicKey.pbData,
                (*pubkey_blob).PublicKey.cbData as usize,
            )
        };

        let mut pubkey_size = 0u32;
        unsafe {
            CryptBinaryToStringA(
                pubkey_data,
                CRYPT_STRING(CRYPT_STRING_BASE64.0 | CRYPT_STRING_NOCR),
                PSTR::null(),
                &mut pubkey_size,
            )
            .ok()?
        };

        let mut pubkey_pem = vec![0u8; pubkey_size as usize];

        unsafe {
            CryptBinaryToStringA(
                pubkey_data,
                CRYPT_STRING(CRYPT_STRING_BASE64.0 | CRYPT_STRING_NOCR),
                PSTR(pubkey_pem.as_mut_ptr()),
                &mut pubkey_size,
            )
            .ok()?
        };

        let pubkey_pem = std::str::from_utf8(&pubkey_pem).map(|s| s.to_string())?;

        Ok(format!(
            "{}\n{}{}",
            "-----BEGIN RSA PUBLIC KEY-----", pubkey_pem, "-----END RSA PUBLIC KEY-----"
        ))
    }

    pub fn private_decrypt(&self, data: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
        let padding_info = BCRYPT_OAEP_PADDING_INFO {
            pszAlgId: BCRYPT_SHA1_ALGORITHM,
            pbLabel: std::ptr::null_mut(),
            cbLabel: 0,
        };

        self.0.private_decrypt(
            data,
            Some(&padding_info as *const _ as *const _),
            BCRYPT_PAD_OAEP,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::Rsa;

    #[test]
    fn debug_rsa() {
        let r = Rsa::generate(4096).unwrap();
        let pub_key = r.public_key().unwrap();
        dbg!(pub_key);
    }
}
