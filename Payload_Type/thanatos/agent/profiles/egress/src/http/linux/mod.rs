#[cfg(feature = "http")]
pub(crate) mod http;

#[cfg(all(feature = "https", feature = "cryptinternal"))]
pub(crate) mod https_internal;

#[cfg(all(feature = "https", not(feature = "cryptinternal")))]
pub(crate) mod https_system;

use std::{
    collections::HashMap,
    io::{BufRead, BufReader, Read},
};

use agent_utils::errors::ProfileInternalError;

#[allow(unused)]
pub(crate) enum HttpVerb {
    POST,
    GET,
}

impl ToString for HttpVerb {
    fn to_string(&self) -> String {
        match self {
            HttpVerb::POST => "POST".to_string(),
            HttpVerb::GET => "GET".to_string(),
        }
    }
}

pub(crate) struct HttpRequest<'a> {
    verb: HttpVerb,
    host: &'a str,
    route: &'a str,
    headers: &'a HashMap<&'static str, &'static str>,
    body: Option<&'a str>,
}

impl<'a> HttpRequest<'a> {
    pub(crate) fn new(
        verb: HttpVerb,
        host: &'a str,
        route: &'a str,
        headers: &'a HashMap<&'static str, &'static str>,
        body: Option<&'a str>,
    ) -> Self {
        Self {
            verb,
            route,
            host,
            headers,
            body,
        }
    }

    pub(crate) fn serialize(&self) -> Vec<u8> {
        let mut serialized = Vec::new();

        serialized.extend_from_slice(
            format!(
                "{} /{} HTTP/1.1\r\nHost: ",
                self.verb.to_string(),
                self.route
            )
            .as_bytes(),
        );

        serialized.extend_from_slice(if let Some(host_header) = self.headers.get("Host") {
            host_header.as_bytes()
        } else {
            let host_header = self.host.trim_start_matches("https://");
            let host_header = host_header.trim_start_matches("http://");
            host_header.as_bytes()
        });

        serialized.extend_from_slice(b"\r\nAccept: */*\r\n");
        serialized.extend_from_slice(b"Connection: close\r\n");

        if let Some(body) = self.body {
            serialized.extend_from_slice(b"Content-Type: application/x-www-form-urlencoded\r\n");
            serialized
                .extend_from_slice(format!("Content-Length: {}\r\n\r\n", body.len()).as_bytes());
            serialized.extend_from_slice(body.as_bytes());
        }

        serialized
    }
}

pub(crate) struct HttpResponse {
    pub status: usize,
    pub data: String,
}

impl HttpResponse {
    fn from_reader(reader: impl std::io::Read) -> Result<Self, ProfileInternalError> {
        let mut buf_reader = BufReader::new(reader);

        let mut headers: HashMap<String, String> = HashMap::new();
        let mut status = 0;

        loop {
            let mut line = String::new();
            buf_reader
                .read_line(&mut line)
                .map_err(|_| ProfileInternalError::Fatal)?;

            line = line.trim_end_matches("\r\n").to_string();

            if line == "" {
                break;
            }

            if line.contains("HTTP/1.1") {
                status = line
                    .split(" ")
                    .nth(1)
                    .ok_or(ProfileInternalError::Fatal)?
                    .parse()
                    .map_err(|_| ProfileInternalError::Fatal)?;
                continue;
            }

            let mut header_entry = line.split(": ");
            let key = header_entry.next().ok_or(ProfileInternalError::Fatal)?;
            let value = header_entry.next().ok_or(ProfileInternalError::Fatal)?;

            headers.insert(key.to_string(), value.to_string());
        }

        if let Some(content_length) = headers.get("Content-Length") {
            let _content_length: usize = content_length
                .parse()
                .map_err(|_| ProfileInternalError::Fatal)?;

            let mut buffer = String::new();

            buf_reader
                .read_to_string(&mut buffer)
                .map_err(|_| ProfileInternalError::Fatal)?;

            Ok(Self {
                status,
                data: buffer,
            })
        } else {
            let mut buffers = Vec::new();
            loop {
                let mut content_length = String::new();
                buf_reader
                    .read_line(&mut content_length)
                    .map_err(|_| ProfileInternalError::Fatal)?;

                content_length = content_length.trim_end_matches("\r\n").to_string();

                let content_length = usize::from_str_radix(content_length.as_str(), 16)
                    .map_err(|_| ProfileInternalError::Fatal)?;

                if content_length == 0 {
                    break;
                }

                let mut buffer = String::new();
                buf_reader
                    .read_line(&mut buffer)
                    .map_err(|_| ProfileInternalError::Fatal)?;

                buffers.push(buffer.trim_end_matches("\r\n").to_string());
            }

            let data = buffers.join("");
            Ok(Self { status, data })
        }
    }
}
