//! Module for handling uuids in string and binary form
use std::str::FromStr;

/// Holds a Uuid
#[repr(transparent)]
#[derive(Debug)]
pub struct Uuid([u8; 16]);

/// Error types when parsing Uuids
#[derive(Debug)]
pub enum UuidError {
    /// The length of the uuid does not equal 16
    InvalidLength,

    /// Uuid contains an invalid character
    InvalidChar(usize),
}

impl From<[u8; 16]> for Uuid {
    fn from(value: [u8; 16]) -> Self {
        Self(value)
    }
}

impl TryFrom<&[u8]> for Uuid {
    type Error = UuidError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        Ok(Uuid::from(
            <[u8; 16]>::try_from(value).map_err(|_| UuidError::InvalidLength)?,
        ))
    }
}

impl FromStr for Uuid {
    type Err = UuidError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.replace('-', "");
        if s.len() != 32 {
            return Err(UuidError::InvalidLength);
        }

        let mut u = [0u8; 16];
        for (uidx, (idx, c)) in s.as_bytes().chunks(2).enumerate().enumerate() {
            if (c[0] < b'0' || c[0] > b'z') || (c[0] > b'9' && c[0] < b'a') {
                return Err(UuidError::InvalidChar(idx));
            }

            let mut msb = c[0] - b'0';
            if msb > 0xf {
                msb -= 39;
            }

            u[uidx] |= msb << 4;

            if (c[1] < b'0' || c[1] > b'z') || (c[1] > b'9' && c[1] < b'a') {
                return Err(UuidError::InvalidChar(idx + 1));
            }

            let mut lsb = c[1] - b'0';
            if lsb > 0xf {
                lsb -= 39;
            }

            u[uidx] |= lsb;
        }

        Ok(Self(u))
    }
}

impl ToString for Uuid {
    fn to_string(&self) -> String {
        let mut s = String::new();
        s.reserve((self.0.len() * 2) + 4);

        for i in 0..4 {
            let mut c = (self.0[i] >> 4) + b'0';
            if c > b'9' {
                c += 39;
            }

            s.push(c as char);

            c = (self.0[i] & 0xf) + b'0';
            if c > b'9' {
                c += 39;
            }

            s.push(c as char);
        }

        s.push('-');

        for byte_chunk in self.0[4..10].chunks(2) {
            for byte in byte_chunk {
                let mut c = (*byte >> 4) + b'0';
                if c > b'9' {
                    c += 39;
                }

                s.push(c as char);

                c = (*byte & 0xf) + b'0';
                if c > b'9' {
                    c += 39;
                }

                s.push(c as char);
            }
            s.push('-');
        }

        for byte in &self.0[10..] {
            let mut c = (byte >> 4) + b'0';
            if c > b'9' {
                c += 39;
            }

            s.push(c as char);

            c = (byte & 0xf) + b'0';
            if c > b'9' {
                c += 39;
            }

            s.push(c as char);
        }

        s
    }
}

impl AsRef<[u8; 16]> for Uuid {
    fn as_ref(&self) -> &[u8; 16] {
        &self.0
    }
}

impl Uuid {
    /// Consumes the Uuid and returns the underlying data
    pub fn into_bytes(self) -> [u8; 16] {
        self.0
    }

    pub fn as_slice(&self) -> &[u8] {
        self.0.as_slice()
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::Uuid;

    #[test]
    fn test_from_str() {
        let uuid = "641d7cc0-706e-4eb2-a051-60193255f914";
        let expected_uuid: [u8; 16] = [
            0x64, 0x1d, 0x7c, 0xc0, 0x70, 0x6e, 0x4e, 0xb2, 0xa0, 0x51, 0x60, 0x19, 0x32, 0x55,
            0xf9, 0x14,
        ];

        let parsed_uuid = Uuid::from_str(uuid).unwrap();
        assert_eq!(parsed_uuid.into_bytes(), expected_uuid);
    }

    #[test]
    fn test_to_string() {
        let uuid: [u8; 16] = [
            137, 88, 246, 26, 79, 248, 73, 16, 154, 144, 40, 181, 36, 20, 209, 76,
        ];

        let expected_uuid = "8958f61a-4ff8-4910-9a90-28b52414d14c";
        let parsed_uuid = Uuid::from(uuid);

        let result_uuid = parsed_uuid.to_string();
        assert_eq!(result_uuid, expected_uuid);
    }

    #[test]
    fn debug_coverage() {
        let uuid_val: [u8; 16] = [
            137, 88, 246, 26, 79, 248, 73, 16, 154, 144, 40, 181, 36, 20, 209, 76,
        ];

        let uuid = Uuid::from(uuid_val);
        dbg!(uuid);
    }

    #[test]
    fn try_from_valid() {
        let uuid_val: [u8; 16] = [0u8; 16];
        <Uuid as TryFrom<&[u8]>>::try_from(uuid_val.as_slice()).unwrap();
    }

    #[test]
    fn try_from_invalid() {
        let uuid_val: [u8; 3] = [0u8; 3];
        assert!(<Uuid as TryFrom<&[u8]>>::try_from(uuid_val.as_slice()).is_err());
    }

    #[test]
    fn uuid_as_ref() {
        let uuid_val: [u8; 16] = [
            137, 88, 246, 26, 79, 248, 73, 16, 154, 144, 40, 181, 36, 20, 209, 76,
        ];

        let uuid = Uuid::from(uuid_val);
        assert_eq!(uuid_val.as_ref(), uuid.as_ref());
    }
}
