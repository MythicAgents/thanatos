use std::{collections::HashMap, error::Error, ffi::c_void, fmt::Write, io::Read};
use windows::{
    core::{HSTRING, PCWSTR},
    Win32::Networking::WinHttp::{
        WinHttpAddRequestHeaders, WinHttpCloseHandle, WinHttpConnect, WinHttpOpen,
        WinHttpOpenRequest, WinHttpQueryDataAvailable, WinHttpReadData, WinHttpReceiveResponse,
        WinHttpSendRequest, WINHTTP_ACCESS_TYPE_AUTOMATIC_PROXY, WINHTTP_ADDREQ_FLAG_ADD,
        WINHTTP_ADDREQ_FLAG_REPLACE, WINHTTP_FLAG_SECURE, WINHTTP_OPEN_REQUEST_FLAGS,
    },
};

use super::profilevars;

#[allow(unused)]
enum HttpVerb {
    Get,
    Post,
}

impl HttpVerb {
    pub fn as_hstring(&self) -> HSTRING {
        match self {
            HttpVerb::Get => HSTRING::from("GET"),
            HttpVerb::Post => HSTRING::from("POST"),
        }
    }
}

#[repr(transparent)]
struct WinHttpSession(*mut c_void);

impl WinHttpSession {
    pub fn new(useragent: &str) -> Result<WinHttpSession, Box<dyn Error>> {
        let useragent = HSTRING::from(useragent);

        let session = unsafe {
            WinHttpOpen(
                &useragent,
                WINHTTP_ACCESS_TYPE_AUTOMATIC_PROXY,
                PCWSTR::null(),
                PCWSTR::null(),
                0,
            )
        };

        if session.is_null() {
            return Err(Box::from(windows::core::Error::from_win32()));
        }

        Ok(Self(session))
    }

    pub fn connect(self, domain: &str, port: u16) -> Result<WinHttpConnectHandle, Box<dyn Error>> {
        let domain = HSTRING::from(domain);
        let connect_handle = unsafe { WinHttpConnect(self.0, &domain, port, 0) };

        if connect_handle.is_null() {
            return Err(Box::from(windows::core::Error::from_win32()));
        }

        Ok(WinHttpConnectHandle {
            handle: connect_handle,
            _session: self,
        })
    }
}

impl Drop for WinHttpSession {
    fn drop(&mut self) {
        unsafe {
            let _ = WinHttpCloseHandle(self.0);
        }
    }
}

struct WinHttpConnectHandle {
    handle: *mut c_void,
    _session: WinHttpSession,
}

impl WinHttpConnectHandle {
    pub fn post(self, uri: &str, tls: bool) -> Result<WinHttpRequestHandle, Box<dyn Error>> {
        self.open_request(HttpVerb::Post, uri, tls)
    }

    #[allow(unused)]
    pub fn get(self, uri: &str, tls: bool) -> Result<WinHttpRequestHandle, Box<dyn Error>> {
        self.open_request(HttpVerb::Get, uri, tls)
    }

    pub fn open_request(
        self,
        verb: HttpVerb,
        uri: &str,
        tls: bool,
    ) -> Result<WinHttpRequestHandle, Box<dyn Error>> {
        let uri = HSTRING::from(uri);

        let secure_flag = if tls {
            WINHTTP_FLAG_SECURE
        } else {
            WINHTTP_OPEN_REQUEST_FLAGS(0)
        };

        let request_handle = unsafe {
            WinHttpOpenRequest(
                self.handle,
                &verb.as_hstring(),
                &uri,
                PCWSTR::null(),
                PCWSTR::null(),
                std::ptr::null(),
                secure_flag,
            )
        };

        if request_handle.is_null() {
            return Err(Box::from(windows::core::Error::from_win32()));
        }

        Ok(WinHttpRequestHandle {
            handle: request_handle,
            _connect: self,
        })
    }
}

impl Drop for WinHttpConnectHandle {
    fn drop(&mut self) {
        unsafe {
            let _ = WinHttpCloseHandle(self.handle);
        }
    }
}

struct WinHttpRequestHandle {
    handle: *mut c_void,
    _connect: WinHttpConnectHandle,
}

impl WinHttpRequestHandle {
    pub fn add_headers(
        self,
        headers: HashMap<String, String>,
    ) -> Result<WinHttpRequestHandle, Box<dyn Error>> {
        if headers.is_empty() {
            return Ok(self);
        }

        let headers = headers
            .into_iter()
            .fold(String::new(), |mut header_string, (key, value)| {
                write!(&mut header_string, "{}: {}\r\n", key, value).unwrap();
                header_string
            });

        let headers = HSTRING::from(headers);
        unsafe {
            WinHttpAddRequestHeaders(
                self.handle,
                headers.as_wide(),
                WINHTTP_ADDREQ_FLAG_ADD | WINHTTP_ADDREQ_FLAG_REPLACE,
            )?
        };

        Ok(self)
    }

    pub fn send_body(self, body: &str) -> Result<WinHttpResponse, Box<dyn Error>> {
        unsafe {
            WinHttpSendRequest(
                self.handle,
                None,
                Some(body.as_ptr().cast()),
                body.len() as u32,
                body.len() as u32,
                0,
            )?
        };

        unsafe { WinHttpReceiveResponse(self.handle, std::ptr::null_mut())? };
        Ok(WinHttpResponse(self))
    }

    #[allow(unused)]
    pub fn send(self) -> Result<WinHttpResponse, Box<dyn Error>> {
        unsafe { WinHttpSendRequest(self.handle, None, None, 0, 0, 0)? };
        unsafe { WinHttpReceiveResponse(self.handle, std::ptr::null_mut())? };
        Ok(WinHttpResponse(self))
    }
}

impl Drop for WinHttpRequestHandle {
    fn drop(&mut self) {
        unsafe {
            let _ = WinHttpCloseHandle(self.handle);
        }
    }
}

struct WinHttpResponse(WinHttpRequestHandle);

impl Read for WinHttpResponse {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let mut read_size = 0u32;
        unsafe {
            WinHttpQueryDataAvailable(self.0.handle, &mut read_size)
                .map_err(|_| std::io::Error::last_os_error())?
        };

        read_size = read_size.min(buf.len() as u32);
        unsafe {
            WinHttpReadData(
                self.0.handle,
                buf.as_mut_ptr().cast(),
                read_size,
                &mut read_size,
            )?
        };
        Ok(read_size as usize)
    }
}

pub fn http_post(host: &str, port: u16, uri: &str, body: &str) -> Result<String, Box<dyn Error>> {
    let secure = host.starts_with("https://");
    let domain = host.split('/').last().unwrap();

    let mut resp = WinHttpSession::new(&profilevars::useragent())?
        .connect(domain, port)?
        .post(uri, secure)?
        .add_headers(profilevars::headers().unwrap_or_default())?
        .send_body(body)?;

    let mut response = String::new();
    resp.read_to_string(&mut response)?;

    Ok(response)
}

#[cfg(test)]
mod tests {
    use std::io::Read;

    use super::WinHttpSession;

    #[test]
    fn winhttp_test() {
        let mut resp = WinHttpSession::new("WinHTTP")
            .unwrap()
            .connect("example.com", 80)
            .unwrap()
            .get("/", false)
            .unwrap()
            .send()
            .unwrap();

        let mut data = String::new();
        resp.read_to_string(&mut data).unwrap();
        println!("{data}");
    }
}
