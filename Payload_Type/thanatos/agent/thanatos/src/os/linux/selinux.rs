use std::io::{BufRead, BufReader};

use errors::ThanatosError;

pub fn selinux_enabled() -> Result<bool, ThanatosError> {
    let f = std::fs::File::open("/proc/self/mountinfo").map_err(ThanatosError::IoError)?;
    let reader = BufReader::new(f);

    for line in reader.lines().map_while(Result::ok) {
        let mut line_split = line.split(' ');

        // According to `proc(5)`, the 9th space delimited item (8th index) contains
        // the filesystem type. If this filesystem type is selinuxfs, it's safe to assume
        // that selinux is present
        if let Some(fstype) = line_split.nth(8) {
            if fstype == "selinuxfs" {
                return Ok(true);
            }
        }
    }

    Ok(false)
}

#[cfg(test)]
mod tests {
    use std::process::Command;

    #[test]
    fn shell_mount() {
        let selinux_found = super::selinux_enabled().expect("Failed to get selinux info");

        let c = Command::new("mount")
            .output()
            .expect("Failed to run 'mount' command");

        let mountinfo = std::str::from_utf8(&c.stdout).expect("Failed to parse 'mount' command");
        assert_eq!(mountinfo.contains("selinuxfs on"), selinux_found);
    }
}
