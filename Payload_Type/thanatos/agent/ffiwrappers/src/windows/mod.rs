use windows::Win32::{
    Foundation::{HMODULE, MAX_PATH},
    System::LibraryLoader::GetModuleFileNameA,
};

use crate::errors::FfiError;

mod hostname;
pub use hostname::hostname;

mod username;
pub use username::username;

mod domain;
pub use domain::domain;

pub mod bcrypt;
mod cffiheaders;
pub mod peb;
pub mod processthreadsapi;
pub mod sysinfoapi;
pub mod winnt;

pub fn process_name() -> Result<String, FfiError> {
    let mut process_path = [0u8; MAX_PATH as usize];
    let path_len = unsafe { GetModuleFileNameA(HMODULE(0), &mut process_path) };

    let process_path = std::str::from_utf8(&process_path[..path_len as usize])
        .map_err(|_| FfiError::StringParseError)?;

    process_path
        .split('\\')
        .last()
        .ok_or(FfiError::StringParseError)
        .map(|s| s.to_string())
}
