#[cfg(unix)]
mod linux;

#[cfg(unix)]
pub use linux::get_entropy;

#[cfg(windows)]
mod windows;

#[cfg(windows)]
pub use windows::get_entropy;

/// Use the CPU rdrand instruction for getting a random value.
/// This is a source of entropy for seeding Xorshiross with the `naive_seed()`
/// function.
///
/// There is a fallback value in case the rdrand CPU feature isn't present. This
/// should be pretty rare since Intel added rdrand in 2012 and AMD added it in 2015.
///
/// A lot of forum posts online mention rdrand has a "spooky secret NSA backdoor".
/// This value isn't used for anything cryptography related. The values from
/// rdrand are being used to generate a seed for Xorshiross and Xorshiross is being
/// used for calculating the sleep jitter value.
///
/// If someone's able to find a flaw in this where they can predict the PRNG
/// state and pre-calculate the agent's sleep interval, good job I guess?
#[cfg(target_arch = "x86_64")]
pub fn try_rdrand() -> usize {
    const FALLBACK_VAL: usize = 0xdeadbeefdeadbeef;
    if std::is_x86_feature_detected!("rdrand") {
        let mut val = 0u64;
        if unsafe { std::arch::x86_64::_rdrand64_step(&mut val) } == 1 {
            val as usize
        } else {
            FALLBACK_VAL
        }
    } else {
        FALLBACK_VAL
    }
}

#[cfg(target_arch = "x86")]
pub fn try_rdrand() -> usize {
    const FALLBACK_VAL: usize = 0xdeadbeef;
    if std::is_x86_feature_detected!("rdrand") {
        let mut val = 0u32;
        if unsafe { std::arch::x86::_rdrand32_step(&mut val) } == 1 {
            val as usize
        } else {
            FALLBACK_VAL
        }
    } else {
        FALLBACK_VAL
    }
}
