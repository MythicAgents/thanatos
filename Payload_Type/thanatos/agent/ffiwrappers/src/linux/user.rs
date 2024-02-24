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

        Ok(Self(NonNull::new(passwd).ok_or_else(FfiError::os_error)?))
    }

    pub fn lookup_uid(uid: u32) -> Result<UserInfo, FfiError> {
        let passwd = unsafe { libc::getpwuid(uid) };

        Ok(Self(NonNull::new(passwd).ok_or_else(FfiError::os_error)?))
    }

    pub fn group_membership(&self) -> Result<GroupMembership, FfiError> {
        let mut ngroups = 0i32;
        if unsafe {
            libc::getgrouplist(
                self.0.as_ref().pw_name,
                self.0.as_ref().pw_gid,
                std::ptr::null_mut(),
                &mut ngroups,
            )
        } != -1
        {
            return Err(FfiError::os_error());
        };

        if ngroups <= 0 {
            return Err(FfiError::os_error());
        }

        let mut gid_list = vec![0u32; ngroups as usize];

        let ret = unsafe {
            libc::getgrouplist(
                self.0.as_ref().pw_name,
                self.gid(),
                gid_list.as_mut_ptr(),
                &mut ngroups,
            )
        };

        if ret != ngroups {
            return Err(FfiError::os_error());
        }

        let members = gid_list
            .into_iter()
            .flat_map(GroupInfo::lookup_gid)
            .collect::<Vec<GroupInfo>>();

        let primary = GroupInfo::current_group().map_err(|_| FfiError::NoGroupMembership)?;

        Ok(GroupMembership { primary, members })
    }

    pub fn username(&self) -> &str {
        unsafe {
            CStr::from_ptr(self.0.as_ref().pw_name)
                .to_str()
                .unwrap_unchecked()
        }
    }

    pub fn passwd(&self) -> &str {
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

    pub fn gecos(&self) -> &str {
        unsafe {
            CStr::from_ptr(self.0.as_ref().pw_gecos)
                .to_str()
                .unwrap_unchecked()
        }
    }

    pub fn home_dir(&self) -> &str {
        unsafe {
            CStr::from_ptr(self.0.as_ref().pw_dir)
                .to_str()
                .unwrap_unchecked()
        }
    }

    pub fn shell(&self) -> &str {
        unsafe {
            CStr::from_ptr(self.0.as_ref().pw_shell)
                .to_str()
                .unwrap_unchecked()
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{
        ffi::CString,
        io::{BufRead, BufReader},
        process::Command,
    };

    #[test]
    /// Compares the effective username with the output of /usr/bin/whoami
    fn whoami_test() {
        let current_user = super::UserInfo::current_user().expect("Failed to get current user");

        let c = Command::new("whoami")
            .output()
            .expect("Failed to run 'whoami'");

        let whoami_output = std::str::from_utf8(&c.stdout)
            .expect("Failed to parse 'whoami' output")
            .trim_end_matches('\n');

        assert_eq!(current_user.username(), whoami_output);
    }

    #[test]
    /// Checks if the group membership info returns successfully
    fn group_success() {
        let current_user = super::UserInfo::current_user().expect("Failed to get current user");
        let res = current_user.group_membership();

        if let Err(e) = res {
            panic!("Group membership failed: {:?}", e);
        }
    }

    #[test]
    fn username_lookup() {
        let root_user = CString::new("root").unwrap();
        let userinfo =
            super::UserInfo::lookup_username(&root_user).expect("Failed to get the root user info");
        assert_eq!(userinfo.uid(), 0);
    }

    #[test]
    fn uid_lookup() {
        let userinfo = super::UserInfo::lookup_uid(0).expect("Failed to get uid 0");
        assert_eq!(userinfo.username(), "root");
    }

    #[test]
    fn shell_passwd() {
        let f = std::fs::File::open("/etc/passwd").expect("Failed to open '/etc/passwd'");
        let reader = BufReader::new(f);
        let userinfo = super::UserInfo::current_user().expect("Failed to get user info");

        let username = userinfo.username();

        for line in reader.lines().map_while(Result::ok) {
            if line.starts_with(username) {
                assert!(line.ends_with(userinfo.shell()));
                return;
            }
        }

        panic!("Failed to find '/etc/passwd' entry for user '{username}'");
    }
}
