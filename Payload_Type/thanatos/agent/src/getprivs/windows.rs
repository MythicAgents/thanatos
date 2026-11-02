use std::os::windows::io::{AsRawHandle, FromRawHandle, OwnedHandle};

use windows::{
    Win32::{
        Foundation::{HANDLE, INVALID_HANDLE_VALUE},
        Security::{
            GetTokenInformation, LUID_AND_ATTRIBUTES, LookupPrivilegeNameA, TOKEN_PRIVILEGES,
            TOKEN_QUERY, TokenPrivileges,
        },
        System::Threading::{GetCurrentProcess, OpenProcessToken},
    },
    core::PSTR,
};

use crate::{agent::AgentTask, mythic_success, utils::platform};

/// Get the privileges of the current process for windows
/// Arguments:
/// * `task` - Tasking information
pub fn get_privileges(task: &AgentTask) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    // Create the initial output
    let mut output = format!(
        "Privileges for '{}' on '{}'\n\n",
        platform::username().unwrap_or_else(|_| "unknown".to_string()),
        platform::hostname().unwrap_or_else(|_| "unknown".to_string()),
    );

    // Get a handle to the current process
    let token_handle = unsafe { OwnedHandle::from_raw_handle(INVALID_HANDLE_VALUE.0.into()) };
    unsafe {
        OpenProcessToken(
            GetCurrentProcess(),
            TOKEN_QUERY,
            &mut HANDLE(token_handle.as_raw_handle()),
        )
    }?;

    // Get the token information length
    let mut priv_len = 0u32;
    let _ = unsafe {
        GetTokenInformation(
            HANDLE(token_handle.as_raw_handle()),
            TokenPrivileges,
            None,
            0,
            &mut priv_len,
        )
    };

    let mut privs = vec![0; priv_len as usize];

    // Get the token information
    unsafe {
        GetTokenInformation(
            HANDLE(token_handle.as_raw_handle()),
            TokenPrivileges,
            Some(privs.as_mut_ptr().cast()),
            priv_len,
            &mut priv_len,
        )
    }?;

    let privileges: &mut TOKEN_PRIVILEGES = unsafe { &mut *privs.as_mut_ptr().cast() };
    let count = privileges.PrivilegeCount;

    // Get the array of LUIDs
    let luids: &mut [LUID_AND_ATTRIBUTES] = unsafe {
        std::slice::from_raw_parts_mut(privileges.Privileges.as_mut_ptr(), count as usize)
    };

    let mut cch_name = [0i8; 512];
    let mut cch_name_size: u32 = cch_name.len() as u32;

    // Iterate over each LUID mapping it to a Windows privilege
    for luid in luids.iter_mut() {
        if unsafe {
            LookupPrivilegeNameA(
                PSTR::null(),
                &mut luid.Luid,
                Some(PSTR(cch_name.as_mut_ptr().cast())),
                &mut cch_name_size,
            )
        }
        .is_ok()
        {
            output.push_str(
                format!("{}\n", unsafe {
                    std::ffi::CStr::from_ptr(cch_name.as_ptr())
                        .to_str()
                        .unwrap()
                })
                .as_str(),
            );

            cch_name.fill(0);
            cch_name_size = cch_name.len() as u32;
        }
    }

    // Return the output
    output = output.trim_end_matches('\n').to_string();
    Ok(mythic_success!(task.id, output))
}
