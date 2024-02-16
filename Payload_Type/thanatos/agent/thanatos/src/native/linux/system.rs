use std::ffi::CString;

use errors::ThanatosError;
use ffiwrappers::{
    errors::FfiError,
    linux::addrinfo::{AddrInfoList, AiFlags, Hints, SockType},
};

#[allow(unused)]
pub fn hostname() -> Result<String, ThanatosError> {
    let h = ffiwrappers::linux::gethostname().map_err(ThanatosError::FFIError)?;
    Ok(h.split('.').next().unwrap_or(&h).to_string())
}

#[allow(unused)]
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

    let canonname = addrlist
        .iter()
        .next()
        .and_then(|addrentry| {
            addrentry
                .canonname()
                .map(|c| c.to_string_lossy().to_string())
        })
        .ok_or(ThanatosError::FFIError(FfiError::CanonNameNotFound))?;

    let mut s = canonname.split('.');
    s.next()
        .ok_or(ThanatosError::FFIError(FfiError::CanonNameNotFound))?;
    Ok(s.collect::<Vec<&str>>().join("."))
}

#[allow(unused)]
pub fn username() -> Result<String, ThanatosError> {
    ffiwrappers::linux::username().map_err(ThanatosError::FFIError)
}

#[cfg(test)]
mod tests {
    use std::ffi::CString;

    use ffiwrappers::linux::addrinfo::{AddrInfoList, AiFlags, Hints, SockType};

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

        let fqdn = format!("{}.{}", host, domain);

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

        let canonname = addrlist
            .iter()
            .next()
            .unwrap()
            .canonname()
            .unwrap()
            .to_string_lossy()
            .to_string();

        assert_eq!(canonname, fqdn);
    }
}
