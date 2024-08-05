use crate::errors::ThanatosError;
use ffiwrappers::linux::user::UserInfo;

pub fn integrity_level() -> Result<u32, ThanatosError> {
    let effective_user = UserInfo::effective_user().map_err(ThanatosError::FfiError)?;
    if effective_user.uid() == 0 {
        return Ok(4);
    }

    let current_groups = UserInfo::current_user()
        .map_err(ThanatosError::FfiError)?
        .group_membership()
        .map_err(ThanatosError::FfiError)?;

    for group in current_groups.members {
        if group.gid() == 0 {
            return Ok(3);
        }

        if group.groupname() == "sudoers" {
            return Ok(3);
        }

        if group.groupname() == "wheel" {
            return Ok(3);
        }
    }

    Ok(2)
}

#[cfg(test)]
mod tests {
    use std::process::Command;

    #[test]
    fn shell_test() {
        let found_integrity = super::integrity_level().expect("Failed to get integrity level");

        let c = Command::new("id")
            .arg("-u")
            .output()
            .expect("Failed to run 'id -u' shell command");

        let id_output = std::str::from_utf8(&c.stdout)
            .expect("Failed to parse 'id -u' output")
            .trim_end_matches('\n');

        let check_uid: u32 = id_output
            .parse()
            .expect("Failed to convert 'id -u' output to an integer");

        if check_uid == 0 && found_integrity == 4 {
            return;
        }

        let c = Command::new("groups")
            .output()
            .expect("Failed to run 'groups' command");

        let group_info = std::str::from_utf8(&c.stdout).expect("Failed to parse command output");

        let shell_group_integrity = if group_info.contains("root") {
            4
        } else if group_info.contains("sudoers") || group_info.contains("wheel") {
            3
        } else {
            2
        };

        assert_eq!(shell_group_integrity, found_integrity);
    }
}
