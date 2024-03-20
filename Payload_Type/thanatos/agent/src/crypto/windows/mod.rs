pub mod aes;
pub mod base64;
pub mod hmac;
pub mod rng;
pub mod rsa;
pub mod sha;

mod bcrypt;

pub mod traits {
    use generic_array::ArrayLength;
    use windows::core::PCWSTR;

    pub(super) mod private {
        pub trait Sealed {}
    }

    pub trait BCryptAlgorithm: private::Sealed {
        const ALGID: PCWSTR;
    }

    pub trait BCryptKeyAlgorithm: BCryptAlgorithm {}
    pub trait BCryptAsymmetricAlgorithm: BCryptKeyAlgorithm {}
    pub trait BCryptSymmetricAlgorithm: BCryptKeyAlgorithm {}

    pub trait BCryptAlgorithmIV: BCryptSymmetricAlgorithm {}

    pub trait BCryptRandomAlgorithm: BCryptAlgorithm {}

    pub trait BCryptHashAlgorithm: BCryptAlgorithm {
        type HashLen: ArrayLength;
    }
}

fn pkcs7_pad(mut data: Vec<u8>, blocksize: u8) -> Vec<u8> {
    let pad_value = if data.len() % blocksize as usize == 0 {
        blocksize
    } else {
        blocksize - ((data.len() % blocksize as usize) as u8)
    };

    data.resize(data.len() + pad_value as usize, pad_value);
    data
}

fn pkcs7_unpad(mut data: Vec<u8>) -> Vec<u8> {
    let pad_value = data[data.len() - 1];
    data.truncate(data.len() - pad_value as usize);
    data
}
