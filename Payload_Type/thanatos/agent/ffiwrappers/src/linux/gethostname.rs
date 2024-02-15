use crate::errors::FfiError;

/// Gets the system's hostname. This is essentially like running `hostname`
pub fn gethostname() -> Result<String, FfiError> {
    // Use `sysconf(3)` to get the max supported hostname length.
    //
    // SAFETY: `sysconf(3)` will return a -1 to signify an error. This is checked below.
    let hostname_max = unsafe { libc::sysconf(libc::_SC_HOST_NAME_MAX) };

    // Check for errors
    if hostname_max == -1 {
        return Err(FfiError::os_error());
    }

    // Cast `hostname_max` into a usize. This can be done safely here without `.try_into()`
    // because `sysconf(3)` uses a signed long to signify an error. The error check
    // is done above so this is guaranteed to not underflow
    let hostname_max = hostname_max as usize;

    // Create a buffer and zero-initialize it to hold the hostname
    let mut hostname_buffer = vec![0u8; hostname_max];

    // Get the system hostname from `gethostname(2)`.
    //
    // SAFETY: This function requires that the pointer to the buffer receiving the hostname
    // is valid and the memory capacity is enough to hold the hostname length.
    // The max hostname length was found above and a buffer was allocated using
    // that maximum length. `gethostname(2)` returns a value of -1 if an error occurs
    if unsafe { libc::gethostname(hostname_buffer.as_mut_ptr().cast(), hostname_max) } == -1 {
        return Err(FfiError::os_error());
    }

    // Find the index of the NULL terminator.
    //
    // According to `gethostname(2)`, the function could truncate the hostname if
    // the buffer is not large enough and is not required to include a null terminator
    // in that instance! A null terminator is needed so that there is no out of bounds
    // read. Return an error if no null terminator is found.
    let null_terminator = hostname_buffer
        .iter()
        .position(|&c| c == 0)
        .ok_or(FfiError::NoNullTerminator)?;

    // Convert the buffer of bytes to a string
    let s = String::from_utf8_lossy(&hostname_buffer[..null_terminator]);
    Ok(s.into_owned())
}

// There's really no way to controllably test this since it relies on a system defined value.
// The test case is mainly for running sanitizers
#[cfg(test)]
mod tests {
    #[test]
    fn gethostname_test() {
        let u = super::gethostname().unwrap();
        dbg!(u);
    }
}
