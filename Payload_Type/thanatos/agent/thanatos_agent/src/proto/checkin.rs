// Automatically generated rust module for 'checkin.proto' file

#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(unused_imports)]
#![allow(unknown_lints)]
#![allow(clippy::all)]
#![cfg_attr(rustfmt, rustfmt_skip)]


use quick_protobuf::{MessageInfo, MessageRead, MessageWrite, BytesReader, Writer, WriterBackend, Result};
use quick_protobuf::sizeofs::*;
use super::*;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Architecture {
    X86 = 0,
    AMD64 = 1,
}

impl Default for Architecture {
    fn default() -> Self {
        Architecture::X86
    }
}

impl From<i32> for Architecture {
    fn from(i: i32) -> Self {
        match i {
            0 => Architecture::X86,
            1 => Architecture::AMD64,
            _ => Self::default(),
        }
    }
}

impl<'a> From<&'a str> for Architecture {
    fn from(s: &'a str) -> Self {
        match s {
            "X86" => Architecture::X86,
            "AMD64" => Architecture::AMD64,
            _ => Self::default(),
        }
    }
}

#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Debug, Default, PartialEq, Clone)]
pub struct CheckinInfo {
    pub ips: Vec<String>,
    pub platform: String,
    pub user: String,
    pub host: String,
    pub pid: u32,
    pub architecture: Architecture,
    pub domain: String,
    pub integrity_level: u32,
    pub process_name: String,
}

impl<'a> MessageRead<'a> for CheckinInfo {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(10) => msg.ips.push(r.read_string(bytes)?.to_owned()),
                Ok(18) => msg.platform = r.read_string(bytes)?.to_owned(),
                Ok(26) => msg.user = r.read_string(bytes)?.to_owned(),
                Ok(34) => msg.host = r.read_string(bytes)?.to_owned(),
                Ok(40) => msg.pid = r.read_uint32(bytes)?,
                Ok(48) => msg.architecture = r.read_enum(bytes)?,
                Ok(58) => msg.domain = r.read_string(bytes)?.to_owned(),
                Ok(64) => msg.integrity_level = r.read_uint32(bytes)?,
                Ok(74) => msg.process_name = r.read_string(bytes)?.to_owned(),
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl MessageWrite for CheckinInfo {
    fn get_size(&self) -> usize {
        0
        + self.ips.iter().map(|s| 1 + sizeof_len((s).len())).sum::<usize>()
        + if self.platform == String::default() { 0 } else { 1 + sizeof_len((&self.platform).len()) }
        + if self.user == String::default() { 0 } else { 1 + sizeof_len((&self.user).len()) }
        + if self.host == String::default() { 0 } else { 1 + sizeof_len((&self.host).len()) }
        + if self.pid == 0u32 { 0 } else { 1 + sizeof_varint(*(&self.pid) as u64) }
        + if self.architecture == checkin::Architecture::X86 { 0 } else { 1 + sizeof_varint(*(&self.architecture) as u64) }
        + if self.domain == String::default() { 0 } else { 1 + sizeof_len((&self.domain).len()) }
        + if self.integrity_level == 0u32 { 0 } else { 1 + sizeof_varint(*(&self.integrity_level) as u64) }
        + if self.process_name == String::default() { 0 } else { 1 + sizeof_len((&self.process_name).len()) }
    }

    fn write_message<W: WriterBackend>(&self, w: &mut Writer<W>) -> Result<()> {
        for s in &self.ips { w.write_with_tag(10, |w| w.write_string(&**s))?; }
        if self.platform != String::default() { w.write_with_tag(18, |w| w.write_string(&**&self.platform))?; }
        if self.user != String::default() { w.write_with_tag(26, |w| w.write_string(&**&self.user))?; }
        if self.host != String::default() { w.write_with_tag(34, |w| w.write_string(&**&self.host))?; }
        if self.pid != 0u32 { w.write_with_tag(40, |w| w.write_uint32(*&self.pid))?; }
        if self.architecture != checkin::Architecture::X86 { w.write_with_tag(48, |w| w.write_enum(*&self.architecture as i32))?; }
        if self.domain != String::default() { w.write_with_tag(58, |w| w.write_string(&**&self.domain))?; }
        if self.integrity_level != 0u32 { w.write_with_tag(64, |w| w.write_uint32(*&self.integrity_level))?; }
        if self.process_name != String::default() { w.write_with_tag(74, |w| w.write_string(&**&self.process_name))?; }
        Ok(())
    }
}

