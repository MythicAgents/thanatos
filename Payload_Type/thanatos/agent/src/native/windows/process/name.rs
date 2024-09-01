use std::path::Path;

use windows::Win32::{
    Foundation::{HMODULE, MAX_PATH},
    System::LibraryLoader::GetModuleFileNameW,
};

use crate::errors::ThanatosError;

pub fn name() -> Result<String, ThanatosError> {
    let mut name_buffer = [0u16; MAX_PATH as usize];
    let len = unsafe { GetModuleFileNameW(HMODULE(0), &mut name_buffer) };

    if len as usize > name_buffer.len() {
        return Err(ThanatosError::last_os_error());
    }

    let file_path = String::from_utf16(&name_buffer[..len as usize])
        .map_err(|_| ThanatosError::InvalidString)?;
    let p = Path::new(&file_path);

    Ok(p.file_name()
        .ok_or(ThanatosError::PathNotAFile)?
        .to_string_lossy()
        .to_string())
}

#[cfg(test)]
mod tests {
    #[test]
    fn process_name() {
        let n = super::name().unwrap();
        dbg!(n);
    }
}
