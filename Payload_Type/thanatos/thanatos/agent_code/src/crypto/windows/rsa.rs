use std::error::Error;

use windows::{
    core::{PCSTR, PCWSTR, PSTR},
    Win32::Security::Cryptography::{
        BCryptCloseAlgorithmProvider, BCryptDecrypt, BCryptDestroyKey, BCryptFinalizeKeyPair,
        BCryptGenerateKeyPair, BCryptOpenAlgorithmProvider, CryptBinaryToStringA,
        CryptExportPublicKeyInfoFromBCryptKeyHandle, BCRYPT_ALG_HANDLE, BCRYPT_KEY_HANDLE,
        BCRYPT_OAEP_PADDING_INFO, BCRYPT_OPEN_ALGORITHM_PROVIDER_FLAGS, BCRYPT_PAD_OAEP,
        BCRYPT_RSA_ALGORITHM, BCRYPT_SHA1_ALGORITHM, CERT_PUBLIC_KEY_INFO,
        CRYPT_OID_INFO_PUBKEY_ENCRYPT_KEY_FLAG, CRYPT_STRING, CRYPT_STRING_BASE64,
        CRYPT_STRING_NOCR, X509_ASN_ENCODING,
    },
};

pub struct Rsa(BCRYPT_KEY_HANDLE);

impl Rsa {
    pub fn generate(bits: u32) -> Result<Rsa, Box<dyn Error>> {
        let mut algorithm_handle = BCRYPT_ALG_HANDLE::default();
        unsafe {
            BCryptOpenAlgorithmProvider(
                &mut algorithm_handle,
                BCRYPT_RSA_ALGORITHM,
                PCWSTR::null(),
                BCRYPT_OPEN_ALGORITHM_PROVIDER_FLAGS(0),
            )
        }
        .ok()?;

        let mut key_handle = BCRYPT_KEY_HANDLE::default();
        unsafe {
            BCryptGenerateKeyPair(algorithm_handle, &mut key_handle, bits, 0)
                .ok()
                .inspect_err(|_| {
                    BCryptCloseAlgorithmProvider(algorithm_handle, 0);
                })?;
        }

        let _ = unsafe { BCryptCloseAlgorithmProvider(algorithm_handle, 0) };

        unsafe { BCryptFinalizeKeyPair(key_handle, 0).ok()? };

        Ok(Self(key_handle))
    }

    pub fn public_key(&self) -> Result<String, Box<dyn Error>> {
        let mut blob_size = 0u32;
        unsafe {
            CryptExportPublicKeyInfoFromBCryptKeyHandle(
                self.0,
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
                self.0,
                X509_ASN_ENCODING,
                PCSTR::null(),
                0,
                None,
                Some(blob_buffer.as_mut_ptr().cast()),
                &mut blob_size,
            )
            .ok()?
        };

        let pubkey_blob = blob_buffer.as_ptr() as *const CERT_PUBLIC_KEY_INFO;

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

        let mut decrypted_length = 0u32;
        unsafe {
            BCryptDecrypt(
                self.0,
                Some(data),
                Some(&padding_info as *const _ as *const _),
                None,
                None,
                &mut decrypted_length,
                BCRYPT_PAD_OAEP,
            )
            .ok()
        }
        .unwrap();

        let mut decrypted_buffer = vec![0u8; decrypted_length as usize];
        unsafe {
            BCryptDecrypt(
                self.0,
                Some(data),
                Some(&padding_info as *const _ as *const _),
                None,
                Some(&mut decrypted_buffer),
                &mut decrypted_length,
                BCRYPT_PAD_OAEP,
            )
            .ok()
        }
        .unwrap();

        Ok(decrypted_buffer)
    }
}

impl Drop for Rsa {
    fn drop(&mut self) {
        unsafe { BCryptDestroyKey(self.0) };
    }
}

#[cfg(test)]
mod tests {
    use std::io::{Read, Write};

    use super::Rsa;

    #[test]
    fn debug_rsa() {
        let r = Rsa::generate(4096).unwrap();
        let pub_key = r.public_key().unwrap();

        let mut kf = std::fs::File::create("pubkey.pem").unwrap();
        kf.write_all(pub_key.as_bytes()).unwrap();

        println!("Waiting.....");
        let mut buff = String::new();
        std::io::stdin().read_line(&mut buff).unwrap();

        let mut f = std::fs::File::open("data.dat").unwrap();
        let mut buf = Vec::new();
        f.read_to_end(&mut buf).unwrap();

        let decrypted = r.private_decrypt(&buf).unwrap();
        let decrypted = std::str::from_utf8(&decrypted).unwrap();
        dbg!(decrypted);
    }
}
