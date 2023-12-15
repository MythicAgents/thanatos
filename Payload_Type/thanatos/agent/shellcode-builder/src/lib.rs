#![no_std]

mod useless;

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    // Don't care about exiting gracefully for now
    unsafe {
        core::ptr::write_volatile(0x4141414141414141 as *mut _, 0);
        core::arch::asm!("hlt", options(noreturn))
    }
}

#[no_mangle]
#[link_section = ".text.entrypoint"]
extern "C" fn entrypoint() {}
