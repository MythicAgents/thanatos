use std::ffi::CString;

use errors::ThanatosError;
use ffiwrappers::{
    errors::FfiError,
    linux::{
        addrinfo::{AddrInfoList, AiFlags, Hints},
        socket::SockType,
    },
};

pub fn hostname() -> Result<String, ThanatosError> {
    let h = ffiwrappers::linux::gethostname().map_err(ThanatosError::FFIError)?;
    Ok(h.split('.').next().unwrap_or(&h).to_string())
}

pub fn domain() -> Result<String, ThanatosError> {
    let current_host = ffiwrappers::linux::gethostname().map_err(ThanatosError::FFIError)?;
    let current_host =
        CString::new(current_host).map_err(|_| ThanatosError::FFIError(FfiError::InteriorNull))?;

    let addrlist = AddrInfoList::with_opts(
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
        .first()
        .canonname()
        .map_err(ThanatosError::FFIError)?
        .to_string();

    let mut s = canonname.split('.');
    s.next()
        .ok_or(ThanatosError::FFIError(FfiError::CanonNameNotFound))?;
    Ok(s.collect::<Vec<&str>>().join("."))
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

        let addrlist = AddrInfoList::with_opts(
            Some(&current_host),
            None,
            Some(Hints {
                socktype: SockType::SockDgram,
                flags: AiFlags::CANONNAME,
                family: Default::default(),
            }),
        )
        .unwrap();

        let mut canonname = addrlist.first().canonname().unwrap().to_string();

        if !canonname.ends_with('.') {
            canonname.push('.');
        }

        assert_eq!(canonname, fqdn);
    }
}
