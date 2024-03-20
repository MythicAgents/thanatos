use windows::{
    Wdk::System::SystemServices::RtlGetVersion, Win32::System::SystemInformation::OSVERSIONINFOEXW,
};

pub fn version() -> String {
    let mut osversioninfo = OSVERSIONINFOEXW::default();
    if unsafe { RtlGetVersion(std::ptr::addr_of_mut!(osversioninfo) as *mut _) }.is_err() {
        return "Windows".to_string();
    }

    format!(
        "Windows {}.{}.{} Build {}",
        osversioninfo.dwMajorVersion,
        osversioninfo.dwMinorVersion,
        osversioninfo.dwBuildNumber,
        osversioninfo.dwBuildNumber
    )
    .to_string()
}

#[cfg(test)]
mod tests {
    #[test]
    fn version_test() {
        let v = super::version();
        println!("{}", v);
    }
}
