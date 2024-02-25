use std::{
    ffi::{CStr, CString},
    mem::MaybeUninit,
};

use crate::errors::FfiError;

const GETGR_SIZE: usize = 1024;

pub struct GroupInfo {
    group: libc::group,
    _buf: Box<[u8; GETGR_SIZE]>,
}

impl GroupInfo {
    pub fn current_group() -> Result<GroupInfo, FfiError> {
        Self::lookup_gid(unsafe { libc::getgid() })
    }

    pub fn effective_group() -> Result<GroupInfo, FfiError> {
        Self::lookup_gid(unsafe { libc::getegid() })
    }

    pub fn lookup_groupname(groupname: &str) -> Result<GroupInfo, FfiError> {
        let mut grpasswd: MaybeUninit<libc::group> = MaybeUninit::uninit();
        let mut grpasswd_ptr = std::ptr::null_mut();
        let mut buf: Box<[u8; GETGR_SIZE]> = Box::new([0u8; GETGR_SIZE]);

        let groupname = CString::new(groupname).map_err(|_| FfiError::InteriorNull)?;

        match unsafe {
            libc::getgrnam_r(
                groupname.as_ptr(),
                grpasswd.as_mut_ptr(),
                buf.as_mut_ptr().cast(),
                buf.len(),
                &mut grpasswd_ptr,
            )
        } {
            0 if !grpasswd_ptr.is_null() => (),
            0 if grpasswd_ptr.is_null() => return Err(FfiError::UserNotFound),
            c => return Err(FfiError::OsError(c)),
        };

        Ok(Self {
            group: unsafe { grpasswd.assume_init() },
            _buf: buf,
        })
    }

    pub fn lookup_gid(gid: u32) -> Result<GroupInfo, FfiError> {
        let mut grpasswd: MaybeUninit<libc::group> = MaybeUninit::uninit();
        let mut grpasswd_ptr = std::ptr::null_mut();
        let mut buf: Box<[u8; GETGR_SIZE]> = Box::new([0u8; GETGR_SIZE]);

        match unsafe {
            libc::getgrgid_r(
                gid,
                grpasswd.as_mut_ptr(),
                buf.as_mut_ptr().cast(),
                buf.len(),
                &mut grpasswd_ptr,
            )
        } {
            0 if !grpasswd_ptr.is_null() => (),
            0 if grpasswd_ptr.is_null() => return Err(FfiError::UserNotFound),
            c => return Err(FfiError::OsError(c)),
        };

        Ok(Self {
            group: unsafe { grpasswd.assume_init() },
            _buf: buf,
        })
    }

    pub fn groupname(&self) -> &str {
        unsafe {
            CStr::from_ptr(self.group.gr_name)
                .to_str()
                .unwrap_unchecked()
        }
    }

    pub fn passwd(&self) -> &str {
        unsafe {
            CStr::from_ptr(self.group.gr_passwd)
                .to_str()
                .unwrap_unchecked()
        }
    }

    pub fn gid(&self) -> u32 {
        self.group.gr_gid
    }
}

#[cfg(test)]
mod tests {
    use std::{
        io::{BufRead, BufReader},
        process::Command,
    };

    use super::GroupInfo;

    #[test]
    fn groups_test() {
        let current_group = GroupInfo::current_group().unwrap();

        let c = Command::new("groups")
            .output()
            .expect("Failed to run 'groups'");

        let groups_output = std::str::from_utf8(&c.stdout)
            .expect("Failed to parse 'groups' output")
            .trim_end_matches('\n');

        let primary_group = groups_output.split(' ').next().unwrap();
        assert_eq!(current_group.groupname(), primary_group);
    }

    #[test]
    fn root_group() {
        let groupname_groupinfo =
            GroupInfo::lookup_groupname("root").expect("Failed to get root group by name");

        let gid_groupinfo = GroupInfo::lookup_gid(0).expect("Failed to get gid 0");

        assert_eq!(groupname_groupinfo.gid(), 0);
        assert_eq!(gid_groupinfo.groupname(), "root");

        assert_eq!(groupname_groupinfo.groupname(), gid_groupinfo.groupname());
        assert_eq!(groupname_groupinfo.passwd(), gid_groupinfo.passwd());
        assert_eq!(groupname_groupinfo.gid(), gid_groupinfo.gid());
    }

    #[test]
    fn group_entry_test() {
        let f = std::fs::File::open("/etc/group").expect("Failed to open '/etc/group'");
        let reader = BufReader::new(f);

        let groupinfo = GroupInfo::current_group().expect("Failed to get group info");
        let groupname = groupinfo.groupname();

        for line in reader.lines().map_while(Result::ok) {
            if line.starts_with(&format!("{}:", groupname)) {
                assert!(line.contains(groupinfo.passwd()));
                assert!(line.contains(&groupinfo.gid().to_string()));
                return;
            }
        }

        panic!("Failed to find '/etc/group' entry for group '{groupname}'");
    }
}
