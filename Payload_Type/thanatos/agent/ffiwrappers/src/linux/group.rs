use std::{ffi::CStr, ptr::NonNull};

use crate::errors::FfiError;

pub struct GroupInfo(NonNull<libc::group>);

impl GroupInfo {
    pub fn current_group() -> Result<GroupInfo, FfiError> {
        Self::lookup_gid(unsafe { libc::getgid() })
    }

    pub fn effective_user() -> Result<GroupInfo, FfiError> {
        Self::lookup_gid(unsafe { libc::getegid() })
    }

    pub fn lookup_username(username: &CStr) -> Result<GroupInfo, FfiError> {
        let grpasswd = unsafe { libc::getgrnam(username.as_ptr()) };
        Ok(Self(
            NonNull::new(grpasswd).ok_or_else(|| FfiError::os_error())?,
        ))
    }

    pub fn lookup_gid(gid: u32) -> Result<GroupInfo, FfiError> {
        let grpasswd = unsafe { libc::getgrgid(gid) };
        Ok(Self(
            NonNull::new(grpasswd).ok_or_else(|| FfiError::os_error())?,
        ))
    }

    pub fn groupname<'a>(&'a self) -> &'a str {
        unsafe {
            CStr::from_ptr(self.0.as_ref().gr_name)
                .to_str()
                .unwrap_unchecked()
        }
    }

    pub fn passwd<'a>(&'a self) -> &'a str {
        unsafe {
            CStr::from_ptr(self.0.as_ref().gr_passwd)
                .to_str()
                .unwrap_unchecked()
        }
    }

    pub fn gid(&self) -> u32 {
        unsafe { self.0.as_ref().gr_gid }
    }
}
