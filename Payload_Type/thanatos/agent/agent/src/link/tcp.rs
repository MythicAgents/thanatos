use serde::Deserialize;

#[derive(Deserialize, Debug)]
#[allow(unused)]
pub(super) struct TcpParameters {
    bind_host: String,
    port: u16,
}
