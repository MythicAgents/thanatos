use std::{ffi::CStr, mem::MaybeUninit, pin::Pin};

use crate::utils::platform::IntegrityLevel;

pub struct PasswdEntry {
    inner: NssEntry<libc::passwd>,
}

impl PasswdEntry {
    pub fn getpwuid(uid: libc::uid_t) -> Result<Option<Self>, std::io::Error> {
        Self::query(|pwd, buf, buflen, result| unsafe {
            libc::getpwuid_r(uid, pwd, buf, buflen, result)
        })
    }

    /// Returns the `pw_name` for the passwd entry
    pub fn name(&self) -> &CStr {
        unsafe { CStr::from_ptr(self.inner.as_ref().pw_name) }
    }

    fn query<F>(f: F) -> Result<Option<Self>, std::io::Error>
    where
        F: Fn(*mut libc::passwd, *mut libc::c_char, usize, *mut *mut libc::passwd) -> libc::c_int,
    {
        NssEntry::query(
            || (unsafe { libc::sysconf(libc::_SC_GETPW_R_SIZE_MAX) }, 256),
            f,
        )
        .map(|entry| entry.map(|entry| Self { inner: entry }))
    }
}

pub struct GroupEntry {
    inner: NssEntry<libc::group>,
}

impl GroupEntry {
    pub fn getgrgid(gid: libc::gid_t) -> Result<Option<Self>, std::io::Error> {
        Self::query(|grp, buf, buflen, result| unsafe {
            libc::getgrgid_r(gid, grp, buf, buflen, result)
        })
    }

    /// Returns the `gr_name` for the passwd entry
    pub fn name(&self) -> &CStr {
        unsafe { CStr::from_ptr(self.inner.as_ref().gr_name) }
    }

    fn query<F>(f: F) -> Result<Option<Self>, std::io::Error>
    where
        F: Fn(*mut libc::group, *mut libc::c_char, usize, *mut *mut libc::group) -> libc::c_int,
    {
        NssEntry::query(
            || (unsafe { libc::sysconf(libc::_SC_GETGR_R_SIZE_MAX) }, 256),
            f,
        )
        .map(|entry| entry.map(|entry| Self { inner: entry }))
    }
}

struct NssEntry<T> {
    entry: T,
    _storage: Pin<Box<[i8]>>,
}

impl<T> NssEntry<T> {
    fn query<S, F>(size_hint: S, query_fn: F) -> Result<Option<Self>, std::io::Error>
    where
        S: FnOnce() -> (i64, usize),
        F: Fn(*mut T, *mut libc::c_char, usize, *mut *mut T) -> libc::c_int,
    {
        let (hint, fallback) = size_hint();
        let mut size = (hint != -1)
            .then(|| usize::try_from(hint.cast_unsigned()).ok())
            .flatten()
            .unwrap_or(fallback);

        let mut entry = MaybeUninit::<T>::zeroed();
        let mut result = std::ptr::null_mut::<T>();
        loop {
            let mut storage = Box::<[i8]>::new_zeroed_slice(size);
            let rc = query_fn(
                entry.as_mut_ptr(),
                storage.as_mut_ptr().cast(),
                storage.len(),
                &mut result,
            );
            if result.is_null() {
                match rc {
                    0 => {
                        return Ok(None);
                    }
                    libc::ERANGE => {
                        size = size
                            .checked_next_power_of_two()
                            .ok_or_else(|| std::io::Error::from_raw_os_error(rc))?;
                    }
                    code => {
                        return Err(std::io::Error::from_raw_os_error(code));
                    }
                }
            } else {
                return Ok(Some(Self {
                    entry: unsafe { entry.assume_init() },
                    _storage: Box::into_pin(unsafe { storage.assume_init() }),
                }));
            }
        }
    }
}

impl<T> AsRef<T> for NssEntry<T> {
    fn as_ref(&self) -> &T {
        &self.entry
    }
}

pub trait IntegrityLevelExt: Sized {
    fn from_uid(uid: libc::uid_t) -> IntegrityLevel;
}

impl IntegrityLevelExt for IntegrityLevel {
    fn from_uid(uid: libc::uid_t) -> IntegrityLevel {
        if uid == 0 {
            IntegrityLevel::High
        } else {
            IntegrityLevel::Medium
        }
    }
}
