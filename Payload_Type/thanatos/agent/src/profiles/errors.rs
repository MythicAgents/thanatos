#[derive(Debug)]
pub enum HttpProfileError {}

#[derive(Debug)]
pub enum HttpConfigError {
    HeadersBase64Invalid,
    HeadersJsonInvalid,
    ProxyInfoBase64Invalid,
    ProxyInfoJsonInvalid,
}
