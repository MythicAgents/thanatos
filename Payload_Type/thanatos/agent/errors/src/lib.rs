use ffiwrappers::errors::FfiError;

#[derive(Debug)]
pub enum ThanatosError {
    OsError(i32),
    FFIError(FfiError),
    NotDomainJoined,

    IoError(std::io::Error),

    ConfigParseError,
}

impl ThanatosError {
    #[cfg(target_os = "windows")]
    pub fn from_windows(e: windows::core::Error) -> Self {
        Self::OsError(e.code().0)
    }
}

#[cfg(test)]
mod tests {
    use crate::ThanatosError;

    #[test]
    fn debug_coverage() {
        let e = ThanatosError::OsError(0);
        dbg!(e);
    }
}
