use errors::ThanatosError;

use windows::{
    core::{Error as WinError, PSTR},
    Win32::{
        Foundation::ERROR_MORE_DATA,
        System::SystemInformation::{ComputerNameDnsDomain, GetComputerNameExA},
    },
};

/// Get the domain name of the system
pub fn domain() -> Result<String, ThanatosError> {
    let mut domainname_length = 0u32;

    // Get the length of the computer's domain name.
    //
    // SAFETY: This will return an error from Windows which needs to be checked.
    // If this is "successful", then the Windows error should contain 'ERROR_MORE_DATA'.
    // This is the error code returned when the buffer is not large enough.
    match unsafe {
        GetComputerNameExA(
            ComputerNameDnsDomain,
            PSTR(std::ptr::null_mut()),
            &mut domainname_length,
        )
    } {
        // Check if 'ERROR_MORE_DATA' was returned
        Err(e) if e.code() == WinError::from(ERROR_MORE_DATA).code() => (),

        // Check if any other error was returned
        Err(e) => return Err(ThanatosError::from_windows(e)),

        // This function should never return successfully since the length is 0
        _ => unreachable!(),
    };

    // Create a buffer for storing the domain name
    //
    // The length can safely be casted to a usize using as since the maximum length
    // of a Windows domain name is 255 characters.
    // ref: https://learn.microsoft.com/en-US/troubleshoot/windows-server/identity/naming-conventions-for-computer-domain-site-ou#dns-domain-names
    let mut domainname_buffer = vec![0u8; domainname_length as usize];

    // Get the computer's domain name.
    //
    // SAFETY: A buffer needs to be allocated for holding the domain name. The
    // length of the domain was found above. The domain name length must match the
    // length of the allocated buffer! An error needs to be checked in case the function fails
    unsafe {
        GetComputerNameExA(
            ComputerNameDnsDomain,
            PSTR(domainname_buffer.as_mut_ptr()),
            &mut domainname_length,
        )
    }
    .map_err(ThanatosError::from_windows)?;

    // Cast the domain name length.
    // The domain name length value now contains the length of the system's domain name
    // without the NULL terminator.
    // ref: https://learn.microsoft.com/en-us/windows/win32/api/sysinfoapi/nf-sysinfoapi-getcomputernameexa
    let domainname_length = domainname_length as usize;

    // Convert the domain name buffer to a string
    let s = String::from_utf8_lossy(&domainname_buffer[..domainname_length]);
    Ok(s.into_owned())
}

#[cfg(test)]
mod tests {
    #[test]
    fn domainname() {
        let domain = super::domain().unwrap();
        dbg!(domain);
    }
}
