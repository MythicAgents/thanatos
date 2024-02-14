use ffiwrappers::windows::bcrypt::{algorithms, BCryptHashHandle};

pub struct Sha256(BCryptHashHandle<algorithms::Sha256>);

impl Sha256 {
    pub fn new() -> Sha256 {
        Sha256(BCryptHashHandle::<algorithms::Sha256>::new())
    }

    pub fn update(&mut self, input: &[u8]) {
        self.0.hash_data(input)
    }

    pub fn finalize(self) -> [u8; 32] {
        self.0.finish_hash().into()
    }
}

impl Default for Sha256 {
    fn default() -> Self {
        Self::new()
    }
}
