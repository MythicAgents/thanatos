#[cfg(target_os = "windows")]
pub use crate::os::windows::{
    build_number, domain, hostname, integrity_level, process_name, product, username,
};

#[cfg(target_os = "linux")]
pub use crate::os::linux::{
    domain, hostname, integrity_level, os_release, platform, process_name, username,
};

use crate::proto::checkin::Architecture;

pub fn architecture() -> Architecture {
    #[cfg(target_arch = "x86_64")]
    let mut arch = Architecture::X8664;

    #[cfg(target_arch = "x86")]
    let mut arch = Architecture::X86;

    #[cfg(target_os = "linux")]
    if let Some(new_arch) = crate::os::linux::architecture() {
        arch = new_arch;
    }

    #[cfg(target_os = "windows")]
    if let Some(new_arch) = crate::os::windows::architecture() {
        arch = new_arch;
    }

    arch
}
