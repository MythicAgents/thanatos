//! Crate which compiles the agent to a cdylib or C library.
//! This is essentially a workaround for https://github.com/rust-lang/rust/issues/51009
//! since Rust will not perform lto on cdylib artifacts if the crate supports outputting cdylib and
//! rlib artifacts
