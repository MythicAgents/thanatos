#[cfg(all(feature = "onload", not(feature = "user")))]
#[ctor::ctor]
pub fn run() {
    let _ = thanatos::real_main();
}

#[cfg(all(not(feature = "onload"), not(feature = "user")))]
#[unsafe(no_mangle)]
extern "system" fn entrypoint() {
    let _ = thanatos::real_main();
}

#[cfg(feature = "user")]
include!(concat!(env!("OUT_DIR"), "/user.rs"));
