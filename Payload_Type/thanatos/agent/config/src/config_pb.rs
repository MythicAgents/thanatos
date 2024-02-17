#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Config {
    #[prost(bytes = "vec", tag = "1")]
    pub uuid: ::prost::alloc::vec::Vec<u8>,
    #[prost(uint32, tag = "2")]
    pub working_hours_start: u32,
    #[prost(uint32, tag = "3")]
    pub working_hours_end: u32,
    #[prost(uint32, tag = "4")]
    pub connection_retries: u32,
    #[prost(bytes = "vec", optional, tag = "5")]
    pub domains: ::core::option::Option<::prost::alloc::vec::Vec<u8>>,
    #[prost(bytes = "vec", optional, tag = "6")]
    pub hostnames: ::core::option::Option<::prost::alloc::vec::Vec<u8>>,
    #[prost(bytes = "vec", optional, tag = "7")]
    pub usernames: ::core::option::Option<::prost::alloc::vec::Vec<u8>>,
    #[prost(string, tag = "8")]
    pub spawn_to: ::prost::alloc::string::String,
    #[prost(message, optional, tag = "9")]
    pub http: ::core::option::Option<HttpConfig>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct HttpConfig {
    #[prost(uint32, tag = "1")]
    pub callback_port: u32,
    #[prost(uint64, tag = "2")]
    pub killdate: u64,
    #[prost(uint32, tag = "3")]
    pub callback_jitter: u32,
    #[prost(map = "string, string", tag = "4")]
    pub headers: ::std::collections::HashMap<
        ::prost::alloc::string::String,
        ::prost::alloc::string::String,
    >,
    #[prost(bytes = "vec", optional, tag = "5")]
    pub aes_key: ::core::option::Option<::prost::alloc::vec::Vec<u8>>,
    #[prost(string, tag = "6")]
    pub callback_host: ::prost::alloc::string::String,
    #[prost(string, tag = "7")]
    pub get_uri: ::prost::alloc::string::String,
    #[prost(string, tag = "8")]
    pub post_uri: ::prost::alloc::string::String,
    #[prost(string, tag = "9")]
    pub query_path_name: ::prost::alloc::string::String,
    #[prost(message, optional, tag = "10")]
    pub proxy: ::core::option::Option<ProxyInfo>,
    #[prost(uint32, tag = "11")]
    pub callback_interval: u32,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ProxyInfo {
    #[prost(string, tag = "1")]
    pub host: ::prost::alloc::string::String,
    #[prost(uint32, tag = "2")]
    pub port: u32,
    #[prost(string, tag = "3")]
    pub pass: ::prost::alloc::string::String,
}
