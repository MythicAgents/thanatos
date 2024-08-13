#[cfg(target_os = "linux")]
pub type Sha256 = openssl::sha::Sha256;

#[cfg(target_os = "windows")]
pub struct Sha256(
    ffiwrappers::windows::bcrypt::BCryptHashHandle<
        ffiwrappers::windows::bcrypt::algorithms::Sha256,
    >,
);

#[cfg(target_os = "windows")]
impl Sha256 {
    pub fn new() -> Sha256 {
        Sha256(BCryptHashHandle::<algorithms::Sha256>::new())
    }

    pub fn update(&mut self, input: &[u8]) {
        // TODO: Make this compatible with openssl Sha256
        self.0.hash_data(input)
    }

    pub fn finish(self) -> [u8; 32] {
        // TODO: Make this compatible with openssl Sha256
        self.0.finish_hash().into()
    }
}

#[cfg(target_os = "windows")]
impl Default for Sha256 {
    fn default() -> Self {
        Self::new()
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
        assert_eq!(h.finish(), expected);
    }

    #[test]
    fn sentence() {
        let s = "let the wind carry you home";

        let expected =
            hex_literal::hex!("2cba8f478e7a181d5541a5e18d8342ef0849b99a11904cf19f08df7b9d3d204c");

        let mut h = Sha256::new();
        h.update(s.as_bytes());
        assert_eq!(h.finish(), expected);
    }
}
