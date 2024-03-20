use serde::Serialize;

use crate::native::{
    architecture::{self, SystemArchitecture},
    domain, host, ips, osinfo, process, user,
};

#[derive(Serialize, Default)]
pub struct CheckinData {
    ips: Vec<String>,
    os: String,
    user: Option<String>,
    host: Option<String>,
    pid: u32,
    architecture: SystemArchitecture,
    domain: Option<String>,
    integrity_level: Option<u32>,
    process_name: Option<String>,
}

pub fn checkin_info() -> CheckinData {
    CheckinData {
        ips: ips::internal_ips().unwrap_or_default(),
        os: osinfo::version(),
        user: user::username().ok(),
        host: host::hostname().ok(),
        pid: std::process::id(),
        architecture: architecture::get_arch(),
        domain: domain::domainname().ok(),
        integrity_level: process::integrity_level().ok(),
        process_name: process::name().ok(),
    }
}
