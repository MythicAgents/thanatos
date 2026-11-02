use std::os::windows::io::{AsRawHandle, BorrowedHandle};

use windows::Win32::{
    Foundation::{ERROR_INSUFFICIENT_BUFFER, ERROR_INVALID_SID, HANDLE},
    Security::{
        GetSidSubAuthority, GetSidSubAuthorityCount, GetTokenInformation, TOKEN_MANDATORY_LABEL,
        TokenIntegrityLevel,
    },
};

use crate::utils::platform::IntegrityLevel;

pub trait IntegrityLevelExt: Sized {
    fn try_from_token(token: BorrowedHandle) -> Result<Self, std::io::Error>;
    fn from_sid_sub_authority(subauth: u32) -> Option<IntegrityLevel>;
}

impl IntegrityLevelExt for IntegrityLevel {
    fn from_sid_sub_authority(subauth: u32) -> Option<IntegrityLevel> {
        let integrity = subauth >> 12;
        match integrity {
            1 => Some(IntegrityLevel::Low),
            2 => Some(IntegrityLevel::Medium),
            3 => Some(IntegrityLevel::High),
            4 => Some(IntegrityLevel::System),
            _ => None,
        }
    }

    fn try_from_token(token: BorrowedHandle) -> Result<Self, std::io::Error> {
        // Grab the token information size
        let mut len = 0u32;
        match unsafe {
            GetTokenInformation(
                HANDLE(token.as_raw_handle()),
                TokenIntegrityLevel,
                None,
                0,
                &mut len,
            )
        } {
            Err(e) if e.code() != ERROR_INSUFFICIENT_BUFFER.to_hresult() => {
                return Err(std::io::Error::from_raw_os_error(e.code().0));
            }
            _ => (),
        }

        let mut buffer =
            Box::<[u8]>::new_uninit_slice(len.try_into().map_err(std::io::Error::other)?);

        // Grab the current process token information
        unsafe {
            GetTokenInformation(
                HANDLE(token.as_raw_handle()),
                TokenIntegrityLevel,
                Some(buffer.as_mut_ptr().cast()),
                len,
                &mut len,
            )?;
        }

        let buffer = unsafe { buffer.assume_init() };

        // Get the integrity level from the token
        let sub_authority: u32 = unsafe {
            let til: &TOKEN_MANDATORY_LABEL = &*buffer.as_ptr().cast();
            let count = (*GetSidSubAuthorityCount(til.Label.Sid)).saturating_sub(1);
            *GetSidSubAuthority(til.Label.Sid, count as u32)
        };

        Self::from_sid_sub_authority(sub_authority)
            .ok_or_else(|| std::io::Error::from_raw_os_error(ERROR_INVALID_SID.to_hresult().0))
    }
}
