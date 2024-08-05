use std::io::ErrorKind;

#[derive(Debug)]
pub enum ThanatosError {
    PastKilldate,
    OutOfProfiles,
    ConfigParse(ConfigParseError),
    IoError(ErrorKind),
    FfiError(ffiwrappers::errors::FfiError),
}

#[derive(Debug)]
pub enum ConfigParseError {
    InvalidUuidLength,
}
