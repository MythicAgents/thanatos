/// Gets the current thread's stack canary value as a source of entropy
#[cfg(target_arch = "x86_64")]
#[inline(always)]
pub fn get_entropy() -> usize {
    let canary = 0usize;
    unsafe { std::arch::asm!("mov {canary}, qword ptr fs:[0x28]", canary = out(reg) canary ) };
    canary
}

/// Gets the current thread's stack canary value as a source of entropy
#[cfg(target_arch = "x86")]
#[inline(always)]
pub fn get_entropy() -> usize {
    let canary = 0usize;
    unsafe { std::arch::asm!("mov {canary}, gs:[0x14]", canary = out(reg) canary ) };
    canary
}
