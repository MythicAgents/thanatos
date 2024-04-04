use std::error::Error;

use windows::{
    core::PSTR,
    Win32::Security::Cryptography::{
        CryptBinaryToStringA, CryptStringToBinaryA, CRYPT_STRING, CRYPT_STRING_BASE64,
        CRYPT_STRING_NOCRLF,
    },
};

pub fn encode(data: impl AsRef<[u8]>) -> String {
    let encoded_len = (((4 * data.as_ref().len().div_ceil(3)) + 3) & !3) + 1;

    let mut encoded_buffer = vec![0u8; encoded_len];
    let mut encoded_len: u32 = encoded_len as u32;

    unsafe {
        CryptBinaryToStringA(
            data.as_ref(),
            CRYPT_STRING(CRYPT_STRING_BASE64.0 | CRYPT_STRING_NOCRLF),
            PSTR(encoded_buffer.as_mut_ptr()),
            &mut encoded_len,
        )
        .ok()
        .unwrap();
    }

    encoded_buffer.truncate(encoded_len as usize);
    std::str::from_utf8(&encoded_buffer).unwrap().to_owned()
}

pub fn decode(data: impl AsRef<str>) -> Result<Vec<u8>, Box<dyn Error>> {
    let padding_len = data
        .as_ref()
        .chars()
        .rev()
        .take_while(|c| c == &'=')
        .count();

    let decoded_len = ((3 * (data.as_ref().len().div_ceil(4))) - padding_len) + 1;

    let mut decoded_buffer = vec![0u8; decoded_len];
    let mut decoded_len: u32 = decoded_len.try_into()?;

    unsafe {
        CryptStringToBinaryA(
            data.as_ref().as_bytes(),
            CRYPT_STRING_BASE64,
            Some(decoded_buffer.as_mut_ptr()),
            &mut decoded_len,
            None,
            None,
        )?
    };

    decoded_buffer.truncate(decoded_len as usize);
    Ok(decoded_buffer)
}

#[cfg(test)]
mod tests {
    const CASES: &[(&str, &str)] = &[
        ("Hello World", "SGVsbG8gV29ybGQ="),
        ("foo", "Zm9v"),
        ("foobar1", "Zm9vYmFyMQ=="),
    ];

    #[test]
    fn encode_test() {
        for (value, expected) in CASES.iter() {
            let encoded = super::encode(*value);
            assert_eq!(encoded.as_str(), *expected);
        }
    }

    #[test]
    fn decode_test() {
        for (expected, value) in CASES.iter() {
            let decoded = super::decode(*value).unwrap();
            assert_eq!(decoded, (*expected).as_bytes());
        }
    }
}
