use std::ffi::CStr;

use errors::ThanatosError;

use super::libc_errno;

pub fn username() -> Result<String, ThanatosError> {
    // Get a pointer to a the `passwd` entry of the current user id
    //
    // SAFETY: This will return NULL on error
    let passwd = unsafe { libc::getpwuid(libc::getuid()) };

    // Check if the returned `passwd` pointer is NULL
    if passwd.is_null() {
        return Err(ThanatosError::OsError(libc_errno()));
    }

    // Get a pointer to the username buffer from the passwd buffer.
    //
    // SAFETY: The `passwd` pointer was checked above to verify that it is not NULL.
    // This should also be NULL checked.
    let username_ptr = unsafe { (*passwd).pw_name };

    // Check if the username pointer is NULL
    if username_ptr.is_null() {
        return Err(ThanatosError::OsError(libc_errno()));
    }

    // Convert the username pointer into a CStr.
    //
    // SAFETY: The `username_ptr` was NULL checked above.
    // This buffer will be a NULL terminated according to `getpwnam(3)`
    let username = unsafe { CStr::from_ptr(username_ptr) };

    Ok(username.to_string_lossy().into_owned())
}

#[cfg(test)]
mod tests {
    #[test]
    fn username_test() {
        let username = super::username().unwrap();
        dbg!(username);
    }
}
