use errors::ThanatosError;

use windows::{
    core::PSTR,
    Win32::{Foundation::ERROR_INSUFFICIENT_BUFFER, System::WindowsProgramming::GetUserNameA},
};

pub fn username() -> Result<String, ThanatosError> {
    let mut username_length = 0u32;

    // Get the length of the current user's username
    //
    // SAFETY: This will return an error from Windows which needs to be checked.
    // If this is "successful", then the Windows error should contain 'ERROR_INSUFFICIENT_BUFFEr'.
    // This is the error code returned when the buffer is not large enough.
    match unsafe { GetUserNameA(PSTR(std::ptr::null_mut()), &mut username_length) } {
        // Check if 'ERROR_MORE_DATA' was returned
        Err(e) if e.code() == windows::core::Error::from(ERROR_INSUFFICIENT_BUFFER).code() => (),

        // Check if any other error was returned
        Err(e) => return Err(ThanatosError::from_windows(e)),

        // This function should never return successfully since the length is 0
        _ => unreachable!(),
    };

    // Create a buffer for storing the username
    //
    // The length can safely be casted to a usize using as since the maximum length
    // of a Windows username is `UNLEN` which is 256 characters.
    // ref: https://learn.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-getusernamea
    let mut username_buffer = vec![0u8; username_length as usize];

    // Get the current user's username.
    //
    // SAFETY: A buffer needs to be allocated for holding the username. The
    // length of the username was found above. The username length must match the
    // length of the allocated buffer! An error needs to be checked in case the function fails
    unsafe { GetUserNameA(PSTR(username_buffer.as_mut_ptr()), &mut username_length) }
        .map_err(ThanatosError::from_windows)?;

    // Cast the username length.
    // The username length value now contains the length of the current user's username
    // including the null terminator
    // ref: https://learn.microsoft.com/en-us/windows/win32/api/sysinfoapi/nf-sysinfoapi-getcomputernameexa
    let username_length = username_length as usize;

    // Convert the username buffer to a string
    let s = String::from_utf8_lossy(&username_buffer[..username_length - 1]);
    Ok(s.into_owned())
}

#[cfg(test)]
mod tests {
    #[test]
    fn username() {
        let username = super::username().unwrap();
        dbg!(username);
    }
}
