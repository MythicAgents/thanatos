use errors::ThanatosError;

/// Gets the system's hostname
pub fn hostname() -> Result<String, ThanatosError> {
    ffiwrappers::windows::hostname()
}
