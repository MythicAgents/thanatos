#![cfg(not(test))]

const CONFIG: &[u8] = include_bytes!(env!("CONFIG"));

pub fn foo() {
    thanatos_http::entrypoint(CONFIG);
}
