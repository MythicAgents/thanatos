use std::mem::MaybeUninit;

use windows::{
    core::PWSTR,
    Win32::System::SystemInformation::{ComputerNameDnsHostname, GetComputerNameExW},
};

use crate::errors::ThanatosError;

pub fn hostname() -> Result<String, ThanatosError> {
    let mut buffer: MaybeUninit<[u16; 256]> = MaybeUninit::uninit();
    let mut buffer_len = std::mem::size_of::<[u16; 256]>() as u32;

    let buffer = unsafe {
        GetComputerNameExW(
            ComputerNameDnsHostname,
            PWSTR(buffer.as_mut_ptr().cast()),
            &mut buffer_len,
        )
        .map_err(|e| ThanatosError::WinError(e.code()))?;
        buffer.assume_init()
    };

    String::from_utf16(&buffer[..buffer_len as usize]).map_err(|_| ThanatosError::InvalidString)
}

#[cfg(test)]
mod tests {
    #[test]
    fn hostname_test() {
        let h = super::hostname().unwrap();
        println!("{}", h);
    }
}
