// TODO: Make this return an enum value for the container environment and return
// it as a separate field in the initial check in
fn check_container_environment() -> Option<&'static str> {
    if let Ok(readdir) = std::fs::read_dir("/") {
        for entry in readdir.flatten() {
            if entry.file_name() == ".dockerenv" {
                return Some("Docker");
            }
        }
    }

    if let Ok(readdir) = std::fs::read_dir("/run") {
        for entry in readdir.flatten() {
            if entry.file_name() == ".containerenv" {
                return Some("Container");
            }
        }
    }

    None
}

// TODO: Return this into a separate initial check in field.
// Parse /proc/self/mountinfo for selinux detection instead of looking for /sys/fs/selinux
fn check_selinux() -> bool {
    if let Ok(readdir) = std::fs::read_dir("/sys/fs") {
        for entry in readdir.flatten() {
            if entry.file_name() == "selinux" {
                return true;
            }
        }
    }

    false
}

// TODO: Split up platform values into separate check in fields and create the platform
// string server side. Also grab the architecture from the initial check in instead
// of embedding it into this string
pub fn platform() -> String {
    let distro = os_release()
        .map(|os_info| {
            os_info
                .pretty_name
                .unwrap_or_else(|| format!("{} {}", os_info.name, os_info.version))
        })
        .unwrap_or_else(|_| "Linux".to_string());

    let utsname = uname::UtsName::new();

    let mut platform_name = match utsname {
        Ok(utsname) => format!(
            "{} kernel {} {}",
            distro,
            utsname.release(),
            utsname.machine()
        )
        .to_string(),
        Err(_) => distro,
    };

    if check_selinux() {
        platform_name.push_str(" (SELinux)");
    }

    if let Some(runtime) = check_container_environment() {
        platform_name.push_str(&format!(" ({runtime})"));
    }

    platform_name
}
