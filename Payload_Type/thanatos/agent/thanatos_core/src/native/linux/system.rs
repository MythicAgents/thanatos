use errors::ThanatosError;

/// Gets the system's hostname
pub fn hostname() -> Result<String, ThanatosError> {
    ffiwrappers::linux::hostname()
}

/// Gets the system's username
pub fn username() -> Result<String, ThanatosError> {
    ffiwrappers::linux::username()
}

pub use super::domain::domains;
