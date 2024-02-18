use std::ffi::CStr;

use crate::errors::FfiError;

pub struct UtsName(libc::utsname);

impl UtsName {
    pub fn new() -> Result<UtsName, FfiError> {
        let mut buf: libc::utsname = unsafe { std::mem::zeroed() };

        if unsafe { libc::uname(&mut buf) } != 0 {
            return Err(FfiError::os_error());
        }

        Ok(Self(buf))
    }

    pub fn sysname(&self) -> &str {
        // SAFETY:
        // - This field will always contain a trailing nullbyte according to `uname(2)`
        //   so the call to `CStr::from_ptr` is safe in this context.
        // - The `to_str()` call will do utf8 validation. The system name will only
        //   ever contain the ascii values "Linux" so there should never be an issue
        //   with the sysname containing invalid utf8 characters
        unsafe {
            CStr::from_ptr(self.0.sysname.as_ptr())
                .to_str()
                .unwrap_unchecked()
        }
    }

    pub fn nodename(&self) -> &str {
        // SAFETY:
        // - This field will always contain a trailing nullbyte.
        // - The node name contains valid ascii characters.
        unsafe {
            CStr::from_ptr(self.0.nodename.as_ptr())
                .to_str()
                .unwrap_unchecked()
        }
    }

    pub fn release(&self) -> &str {
        // SAFETY:
        // - This field will always contain a trailing nullbyte.
        // - The release name will always contain valid ascii characters.
        unsafe {
            CStr::from_ptr(self.0.release.as_ptr())
                .to_str()
                .unwrap_unchecked()
        }
    }

    pub fn version(&self) -> &str {
        // SAFETY:
        // - The version field will contain a trailing nullbyte.
        // - The version contains only valid utf8 data.
        unsafe {
            CStr::from_ptr(self.0.version.as_ptr())
                .to_str()
                .unwrap_unchecked()
        }
    }

    pub fn machine(&self) -> &str {
        // SAFETY:
        // - The machine name will contain a trailing nullbyte.
        // - The machine name contains only valid utf8 data.
        unsafe {
            CStr::from_ptr(self.0.machine.as_ptr())
                .to_str()
                .unwrap_unchecked()
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn sysname() {
        let u = super::UtsName::new().unwrap();
        dbg!(u.sysname());
    }

    #[test]
    fn nodename() {
        let u = super::UtsName::new().unwrap();
        dbg!(u.nodename());
    }

    #[test]
    fn release() {
        let u = super::UtsName::new().unwrap();
        dbg!(u.release());
    }

    #[test]
    fn version() {
        let u = super::UtsName::new().unwrap();
        dbg!(u.version());
    }

    #[test]
    fn machine() {
        let u = super::UtsName::new().unwrap();
        dbg!(u.machine());
    }
}
