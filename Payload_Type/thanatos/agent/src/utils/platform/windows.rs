use std::{
    convert::TryInto,
    ffi::OsString,
    os::windows::{
        ffi::OsStringExt,
        io::{AsHandle, AsRawHandle, FromRawHandle, OwnedHandle},
    },
};

use windows::{
    Win32::{
        Foundation::{ERROR_INSUFFICIENT_BUFFER, HANDLE, INVALID_HANDLE_VALUE},
        Security::TOKEN_QUERY,
        System::{
            SystemInformation::{
                ComputerNameDnsDomain, ComputerNameDnsHostname, GetComputerNameExW,
            },
            Threading::{GetCurrentProcess, OpenProcessToken},
            WindowsProgramming::GetUserNameW,
        },
    },
    core::PWSTR,
};

use crate::utils::{os::windows::IntegrityLevelExt, platform::IntegrityLevel};

pub fn detailed_name() -> Result<String, std::io::Error> {
    Ok(generic_name())
}

pub fn generic_name() -> String {
    "Windows".to_string()
}

pub fn username() -> Result<String, std::io::Error> {
    let mut len = 0;
    // Call `GetUserNameW` to get the username length
    match unsafe { GetUserNameW(None, &mut len) } {
        Err(e) if e.code() != ERROR_INSUFFICIENT_BUFFER.to_hresult() => {
            return Err(std::io::Error::from_raw_os_error(e.code().0));
        }
        _ => (),
    };

    // Allocate memory for the username
    let mut name = Box::<[u16]>::new_uninit_slice(len.try_into().map_err(std::io::Error::other)?);

    // Call `GetUserNameW` to get the current username
    unsafe {
        GetUserNameW(Some(PWSTR(name.as_mut_ptr().cast())), &mut len)?;
    };

    let name = unsafe { name.assume_init() };
    let len = usize::try_from(len).unwrap();

    // Return the username as a String
    Ok(OsString::from_wide(&name[..len])
        .to_string_lossy()
        .to_string())
}

pub fn hostname() -> Result<String, std::io::Error> {
    let mut len = 0;
    // Get the computer name length
    match unsafe { GetComputerNameExW(ComputerNameDnsHostname, None, &mut len) } {
        Err(e) if e.code() != ERROR_INSUFFICIENT_BUFFER.to_hresult() => {
            return Err(std::io::Error::from_raw_os_error(e.code().0));
        }
        _ => (),
    }

    let mut name = Box::<[u16]>::new_uninit_slice(len.try_into().map_err(std::io::Error::other)?);

    // Get the computer hostname
    unsafe {
        GetComputerNameExW(
            ComputerNameDnsHostname,
            Some(PWSTR(name.as_mut_ptr().cast())),
            &mut len,
        )?;
    };

    let name = unsafe { name.assume_init() };
    let len = usize::try_from(len).unwrap();

    // Return the hostname as a String
    Ok(OsString::from_wide(&name[..len])
        .to_string_lossy()
        .to_string())
}

pub fn domain() -> Result<String, std::io::Error> {
    let mut len = 0;
    // Get the domain name length
    match unsafe { GetComputerNameExW(ComputerNameDnsDomain, None, &mut len) } {
        Err(e) if e.code() != ERROR_INSUFFICIENT_BUFFER.to_hresult() => {
            return Err(std::io::Error::from_raw_os_error(e.code().0));
        }
        _ => (),
    }

    let mut name = Box::<[u16]>::new_uninit_slice(len.try_into().map_err(std::io::Error::other)?);

    // Get the domain name
    unsafe {
        GetComputerNameExW(
            ComputerNameDnsDomain,
            Some(PWSTR(name.as_mut_ptr().cast())),
            &mut len,
        )?;
    };

    let name = unsafe { name.assume_init() };
    let len = usize::try_from(len).unwrap();

    // Return the domain name as a String
    Ok(OsString::from_wide(&name[..len])
        .to_string_lossy()
        .to_string())
}

pub fn integrity_level() -> Result<IntegrityLevel, std::io::Error> {
    // Grab the process' token
    let thread_handle = unsafe { OwnedHandle::from_raw_handle(INVALID_HANDLE_VALUE.0) };
    unsafe {
        OpenProcessToken(
            GetCurrentProcess(),
            TOKEN_QUERY,
            &mut HANDLE(thread_handle.as_raw_handle()),
        )?;
    };

    IntegrityLevel::try_from_token(thread_handle.as_handle())
}
