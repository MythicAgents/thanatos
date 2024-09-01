use serde::Serialize;
use windows::Win32::System::SystemInformation::{
    GetNativeSystemInfo, PROCESSOR_ARCHITECTURE_AMD64, PROCESSOR_ARCHITECTURE_INTEL, SYSTEM_INFO,
};

#[derive(Serialize, Default)]
pub enum SystemArchitecture {
    #[serde(rename = "x86")]
    X86,

    #[serde(rename = "x86_64")]
    X86_64,

    #[default]
    #[serde(rename = "UnknownArch")]
    Unknown,
}

pub fn get_arch() -> SystemArchitecture {
    let mut sysinfo = SYSTEM_INFO::default();

    unsafe { GetNativeSystemInfo(&mut sysinfo) };

    if unsafe { sysinfo.Anonymous.Anonymous.wProcessorArchitecture } == PROCESSOR_ARCHITECTURE_AMD64
    {
        SystemArchitecture::X86_64
    } else if unsafe { sysinfo.Anonymous.Anonymous.wProcessorArchitecture }
        == PROCESSOR_ARCHITECTURE_INTEL
    {
        SystemArchitecture::X86
    } else {
        SystemArchitecture::Unknown
    }
}
