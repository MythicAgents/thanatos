use crate::errors::FfiError;
use std::{
    ffi::{CStr, CString},
    mem::MaybeUninit,
};

use super::group::GroupInfo;

const GETPW_SIZE: usize = 1024;

pub struct UserInfo {
    passwd: libc::passwd,
    _buf: Box<[u8; GETPW_SIZE]>,
}

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

    pub fn lookup_username(username: &str) -> Result<UserInfo, FfiError> {
        let mut passwd: MaybeUninit<libc::passwd> = MaybeUninit::uninit();
        let mut passwd_ptr = std::ptr::null_mut();
        let mut buf: Box<[u8; GETPW_SIZE]> = Box::new([0u8; GETPW_SIZE]);

        let username = CString::new(username).map_err(|_| FfiError::InteriorNull)?;

        match unsafe {
            libc::getpwnam_r(
                username.as_ptr(),
                passwd.as_mut_ptr(),
                buf.as_mut_ptr().cast(),
                buf.len(),
                &mut passwd_ptr,
            )
        } {
            0 if !passwd_ptr.is_null() => (),
            0 if passwd_ptr.is_null() => return Err(FfiError::UserNotFound),
            c => return Err(FfiError::OsError(c)),
        };

        Ok(Self {
            passwd: unsafe { passwd.assume_init() },
            _buf: buf,
        })
    }

    pub fn lookup_uid(uid: u32) -> Result<UserInfo, FfiError> {
        let mut passwd: MaybeUninit<libc::passwd> = MaybeUninit::uninit();
        let mut passwd_ptr = std::ptr::null_mut();
        let mut buf: Box<[u8; GETPW_SIZE]> = Box::new([0u8; GETPW_SIZE]);

        match unsafe {
            libc::getpwuid_r(
                uid,
                passwd.as_mut_ptr(),
                buf.as_mut_ptr().cast(),
                buf.len(),
                &mut passwd_ptr,
            )
        } {
            0 if !passwd_ptr.is_null() => (),
            0 if passwd_ptr.is_null() => return Err(FfiError::UserNotFound),
            c => return Err(FfiError::OsError(c)),
        };

        Ok(Self {
            passwd: unsafe { passwd.assume_init() },
            _buf: buf,
        })
    }

    pub fn group_membership(&self) -> Result<GroupMembership, FfiError> {
        let mut ngroups = 0i32;
        if unsafe {
            libc::getgrouplist(
                self.passwd.pw_name,
                self.passwd.pw_gid,
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
                self.passwd.pw_name,
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
            CStr::from_ptr(self.passwd.pw_name)
                .to_str()
                .unwrap_unchecked()
        }
    }

    pub fn passwd(&self) -> &str {
        unsafe {
            CStr::from_ptr(self.passwd.pw_passwd)
                .to_str()
                .unwrap_unchecked()
        }
    }

    pub fn uid(&self) -> u32 {
        self.passwd.pw_uid
    }

    pub fn gid(&self) -> u32 {
        self.passwd.pw_gid
    }

    pub fn gecos(&self) -> &str {
        unsafe {
            CStr::from_ptr(self.passwd.pw_gecos)
                .to_str()
                .unwrap_unchecked()
        }
    }

    pub fn home_dir(&self) -> &str {
        unsafe {
            CStr::from_ptr(self.passwd.pw_dir)
                .to_str()
                .unwrap_unchecked()
        }
    }

    pub fn shell(&self) -> &str {
        unsafe {
            CStr::from_ptr(self.passwd.pw_shell)
                .to_str()
                .unwrap_unchecked()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::UserInfo;
    use std::{
        io::{BufRead, BufReader},
        process::Command,
    };

    #[test]
    fn whoami_test() {
        let current_user = UserInfo::current_user().expect("Failed to get current user");

        let c = Command::new("whoami")
            .output()
            .expect("Failed to run 'whoami'");

        let whoami_output = std::str::from_utf8(&c.stdout)
            .expect("Failed to parse 'whoami' output")
            .trim_end_matches('\n');

        assert_eq!(current_user.username(), whoami_output);
    }

    #[test]
    fn group_success() {
        let current_user = UserInfo::current_user().expect("Failed to get current user");
        let res = current_user.group_membership();

        if let Err(e) = res {
            panic!("Group membership failed: {:?}", e);
        }
    }

    #[test]
    fn root_user() {
        let username_userinfo =
            UserInfo::lookup_username("root").expect("Failed to get the root user by username");

        let uid_userinfo = UserInfo::lookup_uid(0).expect("Failed to get uid 0");

        assert_eq!(username_userinfo.uid(), 0);
        assert_eq!(uid_userinfo.username(), "root");

        assert_eq!(username_userinfo.username(), uid_userinfo.username());
        assert_eq!(username_userinfo.passwd(), uid_userinfo.passwd());
        assert_eq!(username_userinfo.uid(), uid_userinfo.uid());
        assert_eq!(username_userinfo.gid(), uid_userinfo.gid());
        assert_eq!(username_userinfo.gecos(), uid_userinfo.gecos());
        assert_eq!(username_userinfo.home_dir(), uid_userinfo.home_dir());
        assert_eq!(username_userinfo.shell(), uid_userinfo.shell());
    }

    #[test]
    fn passwd_entry_test() {
        let f = std::fs::File::open("/etc/passwd").expect("Failed to open '/etc/passwd'");
        let reader = BufReader::new(f);
        let userinfo = UserInfo::current_user().expect("Failed to get user info");

        let username = userinfo.username();

        for line in reader.lines().map_while(Result::ok) {
            if line.starts_with(&format!("{}:", username)) {
                assert!(line.contains(userinfo.passwd()));
                assert!(line.contains(&userinfo.uid().to_string()));
                assert!(line.contains(&userinfo.gid().to_string()));
                assert!(line.contains(userinfo.gecos()));
                assert!(line.contains(userinfo.home_dir()));
                assert!(line.contains(userinfo.shell()));
                return;
            }
        }

        panic!("Failed to find '/etc/passwd' entry for user '{username}'");
    }
}
