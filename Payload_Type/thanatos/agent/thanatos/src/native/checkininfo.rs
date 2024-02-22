use crate::native::system;
use crate::proto::checkin::{checkin_info::PlatformInfo, CheckinInfo};

#[cfg(target_os = "windows")]
pub fn get_checkininfo(uuid: String) -> CheckinInfo {
    use crate::proto::checkin::WindowsInfo;

    CheckinInfo {
        uuid,
        user: system::username().ok(),
        host: system::hostname().ok(),
        domain: system::domain().ok(),
        pid: Some(std::process::id()),
        architecture: system::architecture().into(),
        platform_info: Some(PlatformInfo::Windows(WindowsInfo {
            build: system::build_number(),
            product: Some(system::product()),
        })),
        integrity_level: system::integrity_level(),
        process_name: system::process_name().ok(),
        ips: Vec::new(),
    }
}
