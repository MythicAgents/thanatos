use errors::ThanatosError;
use ffiwrappers::windows::{processthreadsapi::token::CurrentToken, sysinfoapi};

mod platform;
pub use platform::{build_number, product};

use base_profile::msg::checkin::Architecture;

pub fn username() -> Result<String, ThanatosError> {
    ffiwrappers::windows::username().map_err(ThanatosError::FFIError)
}

pub fn hostname() -> Result<String, ThanatosError> {
    ffiwrappers::windows::hostname().map_err(ThanatosError::FFIError)
}

pub fn domain() -> Result<String, ThanatosError> {
    ffiwrappers::windows::domain().map_err(ThanatosError::FFIError)
}

pub fn process_name() -> Result<String, ThanatosError> {
    ffiwrappers::windows::process_name().map_err(ThanatosError::FFIError)
}

pub fn architecture() -> Option<Architecture> {
    let system_info = sysinfoapi::SystemInfo::new();
    match system_info.processor_architecture() {
        sysinfoapi::ProcessorArchitecture::Amd64 => Some(Architecture::X8664),
        sysinfoapi::ProcessorArchitecture::Intel => Some(Architecture::X86),
        _ => None,
    }
}

pub fn integrity_level() -> Option<u32> {
    let token = CurrentToken::new();
    let sid = token.integrity_level().ok()?;
    let rid = sid.sid().subauthorities().first()?.to_owned();
    Some(rid >> 12)
}
