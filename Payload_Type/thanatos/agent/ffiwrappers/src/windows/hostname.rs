use errors::ThanatosError;

use windows::{
    core::{Error as WinError, PSTR},
    Win32::{
        Foundation::ERROR_MORE_DATA,
        System::SystemInformation::{ComputerNameDnsHostname, GetComputerNameExA},
    },
};

pub fn hostname() -> Result<String, ThanatosError> {
    let mut hostname_length = 0u32;

    // Get the length of the computer's hostname.
    //
    // SAFETY: This will return an error from Windows which needs to be checked.
    // If this is "successful", then the Windows error should contain 'ERROR_MORE_DATA'.
    // This is the error code returned when the buffer is not large enough.
    match unsafe {
        GetComputerNameExA(
            ComputerNameDnsHostname,
            PSTR(std::ptr::null_mut()),
            &mut hostname_length,
        )
    } {
        // Check if 'ERROR_MORE_DATA' was returned
        Err(e) if e.code() == WinError::from(ERROR_MORE_DATA).code() => (),

        // Check if any other error was returned
        Err(e) => return Err(ThanatosError::from_windows(e)),

        // This function should never return successfully since the length is 0
        _ => unreachable!(),
    };

    // Create a buffer for storing the hostname
    //
    // The length can safely be casted to a usize using as since the maximum length
    // of a Windows hostname is 63 characters.
    // ref: https://learn.microsoft.com/en-US/troubleshoot/windows-server/identity/naming-conventions-for-computer-domain-site-ou#dns-host-names
    let mut hostname_buffer = vec![0u8; hostname_length as usize];

    // Get the computer's hostname.
    //
    // SAFETY: A buffer needs to be allocated for holding the hostname. The
    // length of the hostname was found above. The hostname length must match the
    // length of the allocated buffer! An error needs to be checked in case the function fails
    unsafe {
        GetComputerNameExA(
            ComputerNameDnsHostname,
            PSTR(hostname_buffer.as_mut_ptr()),
            &mut hostname_length,
        )
    }
    .map_err(ThanatosError::from_windows)?;

    // Cast the hostname length.
    // The hostname length value now contains the length of the system's hostname
    // without the NULL terminator.
    // ref: https://learn.microsoft.com/en-us/windows/win32/api/sysinfoapi/nf-sysinfoapi-getcomputernameexa
    let hostname_length = hostname_length as usize;

    // Convert the hostname buffer to a string
    let s = String::from_utf8_lossy(&hostname_buffer[..hostname_length]);
    Ok(s.into_owned())
}

#[cfg(test)]
mod tests {
    #[test]
    fn hostname() {
        let hostname = super::hostname().unwrap();
        dbg!(hostname);
    }
}
