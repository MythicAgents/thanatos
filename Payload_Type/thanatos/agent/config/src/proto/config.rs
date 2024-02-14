// Automatically generated rust module for 'config.proto' file

#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(unused_imports)]
#![allow(unknown_lints)]
#![allow(clippy::all)]
#![cfg_attr(rustfmt, rustfmt_skip)]


use std::borrow::Cow;
use std::collections::HashMap;
type KVMap<K, V> = HashMap<K, V>;
use quick_protobuf::{MessageInfo, MessageRead, MessageWrite, BytesReader, Writer, WriterBackend, Result};
use quick_protobuf::sizeofs::*;
use super::*;

#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Debug, Default, PartialEq, Clone)]
pub struct Config<'a> {
    pub uuid: Cow<'a, [u8]>,
    pub working_hours_start: i64,
    pub working_hours_end: i64,
    pub connection_retries: u32,
    pub domains: Cow<'a, [u8]>,
    pub hostnames: Cow<'a, [u8]>,
    pub usernames: Cow<'a, [u8]>,
    pub spawn_to: Cow<'a, str>,
    pub http: Option<HttpConfig<'a>>,
}

impl<'a> MessageRead<'a> for Config<'a> {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(10) => msg.uuid = r.read_bytes(bytes).map(Cow::Borrowed)?,
                Ok(16) => msg.working_hours_start = r.read_int64(bytes)?,
                Ok(24) => msg.working_hours_end = r.read_int64(bytes)?,
                Ok(32) => msg.connection_retries = r.read_uint32(bytes)?,
                Ok(42) => msg.domains = r.read_bytes(bytes).map(Cow::Borrowed)?,
                Ok(50) => msg.hostnames = r.read_bytes(bytes).map(Cow::Borrowed)?,
                Ok(58) => msg.usernames = r.read_bytes(bytes).map(Cow::Borrowed)?,
                Ok(66) => msg.spawn_to = r.read_string(bytes).map(Cow::Borrowed)?,
                Ok(74) => msg.http = Some(r.read_message::<HttpConfig>(bytes)?),
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl<'a> MessageWrite for Config<'a> {
    fn get_size(&self) -> usize {
        0
        + if self.uuid == Cow::Borrowed(b"") { 0 } else { 1 + sizeof_len((&self.uuid).len()) }
        + if self.working_hours_start == 0i64 { 0 } else { 1 + sizeof_varint(*(&self.working_hours_start) as u64) }
        + if self.working_hours_end == 0i64 { 0 } else { 1 + sizeof_varint(*(&self.working_hours_end) as u64) }
        + if self.connection_retries == 0u32 { 0 } else { 1 + sizeof_varint(*(&self.connection_retries) as u64) }
        + if self.domains == Cow::Borrowed(b"") { 0 } else { 1 + sizeof_len((&self.domains).len()) }
        + if self.hostnames == Cow::Borrowed(b"") { 0 } else { 1 + sizeof_len((&self.hostnames).len()) }
        + if self.usernames == Cow::Borrowed(b"") { 0 } else { 1 + sizeof_len((&self.usernames).len()) }
        + if self.spawn_to == "" { 0 } else { 1 + sizeof_len((&self.spawn_to).len()) }
        + self.http.as_ref().map_or(0, |m| 1 + sizeof_len((m).get_size()))
    }

    fn write_message<W: WriterBackend>(&self, w: &mut Writer<W>) -> Result<()> {
        if self.uuid != Cow::Borrowed(b"") { w.write_with_tag(10, |w| w.write_bytes(&**&self.uuid))?; }
        if self.working_hours_start != 0i64 { w.write_with_tag(16, |w| w.write_int64(*&self.working_hours_start))?; }
        if self.working_hours_end != 0i64 { w.write_with_tag(24, |w| w.write_int64(*&self.working_hours_end))?; }
        if self.connection_retries != 0u32 { w.write_with_tag(32, |w| w.write_uint32(*&self.connection_retries))?; }
        if self.domains != Cow::Borrowed(b"") { w.write_with_tag(42, |w| w.write_bytes(&**&self.domains))?; }
        if self.hostnames != Cow::Borrowed(b"") { w.write_with_tag(50, |w| w.write_bytes(&**&self.hostnames))?; }
        if self.usernames != Cow::Borrowed(b"") { w.write_with_tag(58, |w| w.write_bytes(&**&self.usernames))?; }
        if self.spawn_to != "" { w.write_with_tag(66, |w| w.write_string(&**&self.spawn_to))?; }
        if let Some(ref s) = self.http { w.write_with_tag(74, |w| w.write_message(s))?; }
        Ok(())
    }
}

#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Debug, Default, PartialEq, Clone)]
pub struct HttpConfig<'a> {
    pub callback_port: u32,
    pub killdate: u64,
    pub callback_jitter: u32,
    pub headers: KVMap<Cow<'a, str>, Cow<'a, str>>,
    pub aes_key: Cow<'a, [u8]>,
    pub callback_host: Cow<'a, str>,
    pub get_uri: Cow<'a, str>,
    pub post_uri: Cow<'a, str>,
    pub query_path_name: Cow<'a, str>,
    pub proxy: Option<ProxyInfo<'a>>,
    pub callback_interval: u32,
}

impl<'a> MessageRead<'a> for HttpConfig<'a> {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(8) => msg.callback_port = r.read_uint32(bytes)?,
                Ok(16) => msg.killdate = r.read_uint64(bytes)?,
                Ok(24) => msg.callback_jitter = r.read_uint32(bytes)?,
                Ok(34) => {
                    let (key, value) = r.read_map(bytes, |r, bytes| Ok(r.read_string(bytes).map(Cow::Borrowed)?), |r, bytes| Ok(r.read_string(bytes).map(Cow::Borrowed)?))?;
                    msg.headers.insert(key, value);
                }
                Ok(42) => msg.aes_key = r.read_bytes(bytes).map(Cow::Borrowed)?,
                Ok(50) => msg.callback_host = r.read_string(bytes).map(Cow::Borrowed)?,
                Ok(58) => msg.get_uri = r.read_string(bytes).map(Cow::Borrowed)?,
                Ok(66) => msg.post_uri = r.read_string(bytes).map(Cow::Borrowed)?,
                Ok(74) => msg.query_path_name = r.read_string(bytes).map(Cow::Borrowed)?,
                Ok(82) => msg.proxy = Some(r.read_message::<ProxyInfo>(bytes)?),
                Ok(88) => msg.callback_interval = r.read_uint32(bytes)?,
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl<'a> MessageWrite for HttpConfig<'a> {
    fn get_size(&self) -> usize {
        0
        + if self.callback_port == 0u32 { 0 } else { 1 + sizeof_varint(*(&self.callback_port) as u64) }
        + if self.killdate == 0u64 { 0 } else { 1 + sizeof_varint(*(&self.killdate) as u64) }
        + if self.callback_jitter == 0u32 { 0 } else { 1 + sizeof_varint(*(&self.callback_jitter) as u64) }
        + self.headers.iter().map(|(k, v)| 1 + sizeof_len(2 + sizeof_len((k).len()) + sizeof_len((v).len()))).sum::<usize>()
        + if self.aes_key == Cow::Borrowed(b"") { 0 } else { 1 + sizeof_len((&self.aes_key).len()) }
        + if self.callback_host == "" { 0 } else { 1 + sizeof_len((&self.callback_host).len()) }
        + if self.get_uri == "" { 0 } else { 1 + sizeof_len((&self.get_uri).len()) }
        + if self.post_uri == "" { 0 } else { 1 + sizeof_len((&self.post_uri).len()) }
        + if self.query_path_name == "" { 0 } else { 1 + sizeof_len((&self.query_path_name).len()) }
        + self.proxy.as_ref().map_or(0, |m| 1 + sizeof_len((m).get_size()))
        + if self.callback_interval == 0u32 { 0 } else { 1 + sizeof_varint(*(&self.callback_interval) as u64) }
    }

    fn write_message<W: WriterBackend>(&self, w: &mut Writer<W>) -> Result<()> {
        if self.callback_port != 0u32 { w.write_with_tag(8, |w| w.write_uint32(*&self.callback_port))?; }
        if self.killdate != 0u64 { w.write_with_tag(16, |w| w.write_uint64(*&self.killdate))?; }
        if self.callback_jitter != 0u32 { w.write_with_tag(24, |w| w.write_uint32(*&self.callback_jitter))?; }
        for (k, v) in self.headers.iter() { w.write_with_tag(34, |w| w.write_map(2 + sizeof_len((k).len()) + sizeof_len((v).len()), 10, |w| w.write_string(&**k), 18, |w| w.write_string(&**v)))?; }
        if self.aes_key != Cow::Borrowed(b"") { w.write_with_tag(42, |w| w.write_bytes(&**&self.aes_key))?; }
        if self.callback_host != "" { w.write_with_tag(50, |w| w.write_string(&**&self.callback_host))?; }
        if self.get_uri != "" { w.write_with_tag(58, |w| w.write_string(&**&self.get_uri))?; }
        if self.post_uri != "" { w.write_with_tag(66, |w| w.write_string(&**&self.post_uri))?; }
        if self.query_path_name != "" { w.write_with_tag(74, |w| w.write_string(&**&self.query_path_name))?; }
        if let Some(ref s) = self.proxy { w.write_with_tag(82, |w| w.write_message(s))?; }
        if self.callback_interval != 0u32 { w.write_with_tag(88, |w| w.write_uint32(*&self.callback_interval))?; }
        Ok(())
    }
}

#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Debug, Default, PartialEq, Clone)]
pub struct ProxyInfo<'a> {
    pub host: Cow<'a, str>,
    pub port: u32,
    pub pass: Cow<'a, str>,
}

impl<'a> MessageRead<'a> for ProxyInfo<'a> {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(10) => msg.host = r.read_string(bytes).map(Cow::Borrowed)?,
                Ok(16) => msg.port = r.read_uint32(bytes)?,
                Ok(26) => msg.pass = r.read_string(bytes).map(Cow::Borrowed)?,
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl<'a> MessageWrite for ProxyInfo<'a> {
    fn get_size(&self) -> usize {
        0
        + if self.host == "" { 0 } else { 1 + sizeof_len((&self.host).len()) }
        + if self.port == 0u32 { 0 } else { 1 + sizeof_varint(*(&self.port) as u64) }
        + if self.pass == "" { 0 } else { 1 + sizeof_len((&self.pass).len()) }
    }

    fn write_message<W: WriterBackend>(&self, w: &mut Writer<W>) -> Result<()> {
        if self.host != "" { w.write_with_tag(10, |w| w.write_string(&**&self.host))?; }
        if self.port != 0u32 { w.write_with_tag(16, |w| w.write_uint32(*&self.port))?; }
        if self.pass != "" { w.write_with_tag(26, |w| w.write_string(&**&self.pass))?; }
        Ok(())
    }
}

