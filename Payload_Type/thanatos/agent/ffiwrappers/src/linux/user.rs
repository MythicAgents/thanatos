use crate::errors::FfiError;
use std::{ffi::CStr, ptr::NonNull};

use super::group::GroupInfo;

pub struct UserInfo(NonNull<libc::passwd>);

pub struct GroupMembership {
    pub primary: GroupInfo,
    pub members: Vec<GroupInfo>,
}

impl UserInfo {
    pub fn current_user() -> Result<UserInfo, FfiError> {
        Self::lookup_uid(unsafe { libc::getuid() })
    }

    pub fn effective_user() -> Result<UserInfo, FfiError> {
        Self::lookup_uid(unsafe { libc::geteuid() })
    }

    pub fn lookup_username(username: &CStr) -> Result<UserInfo, FfiError> {
        let passwd = unsafe { libc::getpwnam(username.as_ptr()) };

        Ok(Self(
            NonNull::new(passwd).ok_or_else(|| FfiError::os_error())?,
        ))
    }

    pub fn lookup_uid(uid: u32) -> Result<UserInfo, FfiError> {
        let passwd = unsafe { libc::getpwuid(uid) };

        Ok(Self(
            NonNull::new(passwd).ok_or_else(|| FfiError::os_error())?,
        ))
    }

    pub fn group_membership(&self) -> Result<GroupMembership, FfiError> {
        let mut ngroups = 0i32;
        unsafe {
            libc::getgrouplist(
                self.0.as_ref().pw_name,
                self.0.as_ref().pw_gid,
                std::ptr::null_mut(),
                &mut ngroups,
            )
        };

        if ngroups <= 0 {
            return Err(FfiError::os_error());
        }

        let mut gid_list = vec![0u32; ngroups as usize];

        if unsafe {
            libc::getgrouplist(
                self.0.as_ref().pw_name,
                self.gid(),
                gid_list.as_mut_ptr(),
                &mut ngroups,
            )
        } != 0
        {
            return Err(FfiError::os_error());
        }

        let members = gid_list
            .into_iter()
            .flat_map(|gid| GroupInfo::lookup_gid(gid))
            .collect::<Vec<GroupInfo>>();

        let primary = GroupInfo::current_group().map_err(|_| FfiError::NoGroupMembership)?;

        Ok(GroupMembership { primary, members })
    }

    pub fn username<'a>(&'a self) -> &'a str {
        unsafe {
            CStr::from_ptr(self.0.as_ref().pw_name)
                .to_str()
                .unwrap_unchecked()
        }
    }

    pub fn passwd<'a>(&'a self) -> &'a str {
        unsafe {
            CStr::from_ptr(self.0.as_ref().pw_passwd)
                .to_str()
                .unwrap_unchecked()
        }
    }

    pub fn uid(&self) -> u32 {
        unsafe { self.0.as_ref().pw_uid }
    }

    pub fn gid(&self) -> u32 {
        unsafe { self.0.as_ref().pw_gid }
    }

    pub fn gecos<'a>(&'a self) -> &'a str {
        unsafe {
            CStr::from_ptr(self.0.as_ref().pw_gecos)
                .to_str()
                .unwrap_unchecked()
        }
    }

    pub fn home_dir<'a>(&'a self) -> &'a str {
        unsafe {
            CStr::from_ptr(self.0.as_ref().pw_dir)
                .to_str()
                .unwrap_unchecked()
        }
    }

    pub fn shell<'a>(&'a self) -> &'a str {
        unsafe {
            CStr::from_ptr(self.0.as_ref().pw_shell)
                .to_str()
                .unwrap_unchecked()
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    pub fn info() {
        let current_user = super::UserInfo::current_user().unwrap();

        dbg!(current_user.username());
        dbg!(current_user.passwd());
        dbg!(current_user.uid());
        dbg!(current_user.gid());
        dbg!(current_user.gecos());
        dbg!(current_user.home_dir());
        dbg!(current_user.shell());
    }
}
