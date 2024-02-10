pub struct Sha256;

impl Sha256 {
    pub fn new() -> Sha256 {
        Sha256
    }

    pub fn update(&mut self, input: &[u8]) {}

    pub fn finalize(self) -> [u8; 32] {
        [0u8; 32]
    }
}
