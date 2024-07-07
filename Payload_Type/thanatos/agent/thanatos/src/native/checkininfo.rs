use thanatos_protos::msg::checkin::CheckinData;

#[cfg(target_os = "windows")]
pub fn get_checkininfo() -> CheckinData {
    use crate::{native, os::windows};
    use base_profile::msg::checkin::WindowsInfo;

    CheckinData {
        user: windows::username().ok(),
        host: windows::hostname().ok(),
        domain: windows::domain().ok(),
        pid: Some(std::process::id()),
        architecture: native::architecture().into(),
        platform_info: Some(PlatformInfo::Windows(WindowsInfo {
            build: windows::build_number(),
            product: Some(windows::product()),
        })),
        integrity_level: windows::integrity_level(),
        process_name: windows::process_name().ok(),
        ips: windows::internal_ips().unwrap_or_default(),
    }
}

#[cfg(target_os = "linux")]
pub fn get_checkininfo() -> CheckinData {
    use thanatos_protos::msg::checkin::{checkin_data::PlatformInfo, LinuxInfo};

    use crate::{native, os::linux};

    CheckinData {
        user: linux::username().ok(),
        host: linux::hostname().ok(),
        domain: linux::domain().ok(),
        pid: Some(std::process::id()),
        architecture: native::architecture().into(),
        integrity_level: linux::integrity_level().ok(),
        process_name: linux::process_name().ok(),
        platform_info: Some(PlatformInfo::Linux(LinuxInfo {
            distro: linux::distro(),
            kernel: linux::kernel(),
            selinux: linux::selinux_enabled().unwrap_or(false),
            container: linux::container_environment().into(),
        })),
        ips: linux::internal_ips().unwrap_or_default(),
    }
}
