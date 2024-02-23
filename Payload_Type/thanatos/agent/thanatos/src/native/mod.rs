use crate::proto::checkin::Architecture;

pub mod checkininfo;

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
