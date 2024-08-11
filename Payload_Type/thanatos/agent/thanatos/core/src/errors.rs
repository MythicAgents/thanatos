use std::io::ErrorKind;

#[derive(Debug)]
pub enum ThanatosError {
    PastKilldate,
    OutOfProfiles,
    ConfigParse(ConfigParseError),
    #[allow(dead_code)]
    IoError(ErrorKind),
    #[allow(dead_code)]
    FfiError(ffiwrappers::errors::FfiError),
}

#[derive(Debug)]
pub enum ConfigParseError {
    InvalidUuidLength,
}
