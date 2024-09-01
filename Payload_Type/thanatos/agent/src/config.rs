#[derive(Default)]
pub struct Config {
    pub uuid: String,
    pub connection_retries: usize,
    pub tlsuntrusted: bool,
    pub working_start: String,
    pub working_end: String,
    pub systemproxy: bool,
    pub callback_port: u16,
    pub killdate: u64,
    pub eke: bool,
    pub callback_jitter: u32,
    pub headers: Option<String>,
    pub aes_key: Option<String>,
    pub callback_host: String,
    pub get_uri: String,
    pub post_uri: String,
    pub query_path_name: String,
    pub proxy_info: Option<String>,
    pub callback_interval: u32,
    #[cfg(unix)]
    pub daemonize: bool,
}
