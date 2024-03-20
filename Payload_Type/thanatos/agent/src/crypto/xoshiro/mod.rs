//! [Xoshiro++](https://prng.di.unimi.it/) module for quick PRNG generation.
//! Implementation based off of the [xoshiro256starstar.c](https://prng.di.unimi.it/xoshiro256starstar.c)
//! example.

mod helpers;
use helpers::get_entropy;

use self::helpers::try_rdrand;

#[inline(always)]
fn rotl(x: usize, k: i32) -> usize {
    (x << k) | (x >> (2i32.pow(std::mem::size_of::<usize>() as u32) - k))
}

/// [Xoshiro**](https://prng.di.unimi.it/) PRNG
///
/// This PRNG is NOT cryptographically secure and should NEVER be used for crypto
/// routines under any circumstances!!!
///
/// Xoshiro++ is being used for calculating the sleep jitter value and the
/// ephemeral session id value for registering the key exchange with Mythic.
/// This provides a reasonably good PRNG
#[derive(Debug)]
pub struct Xoshiross {
    state: [usize; 4],
}

impl Xoshiross {
    /// Naive seeded Xorshiro++. A secure seed is not really important because
    /// this rng will only be used for calculating the sleep jitter and creating
    /// an ephemeral session id for the key exchange.
    ///
    /// Xoshiro++ is NOT a CSPRNG so this should never be used for crypto under
    /// ANY circumstances!!!!
    pub fn naive_seed() -> Xoshiross {
        let mut seed: [usize; 4] = [0; 4];

        let entropy = get_entropy();
        seed[0] = entropy ^ try_rdrand();

        let mut shuffled = entropy;
        shuffled ^= shuffled << 13;
        shuffled ^= shuffled >> 7;
        shuffled ^= shuffled << 17;
        seed[1] = shuffled;

        shuffled = seed[1] ^ try_rdrand();
        shuffled ^= shuffled << 13;
        shuffled ^= shuffled >> 7;
        shuffled ^= shuffled << 17;
        seed[2] = shuffled;

        shuffled = usize::from_be_bytes(seed[2].to_le_bytes()) ^ try_rdrand();
        shuffled ^= shuffled << 13;
        shuffled ^= shuffled >> 7;
        shuffled ^= shuffled << 17;
        seed[3] = shuffled;

        Xoshiross { state: seed }
    }

    pub fn next_val(&mut self) -> usize {
        let result = rotl(self.state[1] * 5, 7) * 9;
        let t = self.state[1] << 17;

        self.state[2] ^= self.state[0];
        self.state[3] ^= self.state[1];
        self.state[1] ^= self.state[2];
        self.state[0] ^= self.state[3];

        self.state[2] ^= t;
        self.state[3] = rotl(self.state[3], 45);

        result
    }

    pub fn gen_bytes<const N: usize>(&mut self) -> [u8; N] {
        let mut result = [0u8; N];

        for chunk in result.chunks_exact_mut(std::mem::size_of::<usize>()) {
            chunk.copy_from_slice(&self.next_val().to_be_bytes());
        }

        result
            .chunks_exact_mut(std::mem::size_of::<usize>())
            .into_remainder()
            .copy_from_slice(&self.next_val().to_be_bytes());

        result
    }
}

#[cfg(test)]
mod tests {
    use super::Xoshiross;

    #[test]
    fn seed_test() {
        let x = Xoshiross::naive_seed();
        println!("{:x?}", x);
    }
}
