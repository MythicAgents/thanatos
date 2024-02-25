#![allow(clippy::new_without_default)]
use sha2::Digest;

#[repr(transparent)]
pub struct Sha256(sha2::Sha256);

impl Sha256 {
    pub fn new() -> Sha256 {
        Sha256(sha2::Sha256::new())
    }

    pub fn update(&mut self, input: &[u8]) {
        self.0.update(input);
    }

    pub fn finalize(self) -> [u8; 32] {
        self.0.finalize().into()
    }
}

#[cfg(test)]
mod tests {
    use super::Sha256;

    #[test]
    fn simple_word() {
        let w = "hello";

        let expected =
            hex_literal::hex!("2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824");

        let mut h = Sha256::new();
        h.update(w.as_bytes());
        assert_eq!(h.finalize(), expected);
    }

    #[test]
    fn sentence() {
        let s = "let the wind carry you home";

        let expected =
            hex_literal::hex!("2cba8f478e7a181d5541a5e18d8342ef0849b99a11904cf19f08df7b9d3d204c");

        let mut h = Sha256::new();
        h.update(s.as_bytes());
        assert_eq!(h.finalize(), expected);
    }
}
