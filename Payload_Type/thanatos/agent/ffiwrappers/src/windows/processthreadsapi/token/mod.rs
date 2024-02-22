pub mod access;

use std::marker::PhantomData;

use access::TokenCurrent;
use windows::{
    core::HRESULT,
    Win32::{
        Foundation::HANDLE,
        Security::{GetTokenInformation, TokenIntegrityLevel},
    },
};

use traits::*;

use crate::{errors::FfiError, windows::winnt::SidAndAttributes};

pub mod traits {
    use crate::internal::SealedTrait;
    use windows::Win32::Security::TOKEN_ACCESS_MASK;

    pub trait TokenAccessRights: SealedTrait {
        const MASK: TOKEN_ACCESS_MASK;
    }

    pub trait TokenQueryRights: TokenAccessRights {}
    pub trait TokenQuerySourceRights: TokenAccessRights {}
}

#[repr(transparent)]
pub(crate) struct TokenInner<AccessRights: TokenAccessRights> {
    pub handle: HANDLE,
    pub _access: PhantomData<AccessRights>,
}

#[repr(transparent)]
pub struct CurrentToken(TokenInner<TokenCurrent>);

#[repr(transparent)]
pub struct Token<AccessRights: TokenAccessRights>(pub(crate) TokenInner<AccessRights>);

impl CurrentToken {
    pub const fn new() -> CurrentToken {
        CurrentToken(TokenInner {
            handle: HANDLE(-4),
            _access: PhantomData,
        })
    }

    pub fn integrity_level(&self) -> Result<SidAndAttributes, FfiError> {
        self.0.integrity_level()
    }
}

impl<AccessRights: TokenAccessRights> Token<AccessRights> {
    pub const fn current_token() -> CurrentToken {
        CurrentToken::new()
    }
}

impl<AccessRights: TokenQueryRights> Token<AccessRights> {
    pub fn integrity_level(&self) -> Result<SidAndAttributes, FfiError> {
        self.0.integrity_level()
    }
}

impl<AccessRights: TokenQueryRights> TokenInner<AccessRights> {
    pub fn integrity_level(&self) -> Result<SidAndAttributes, FfiError> {
        let mut token_info_length = 0u32;

        match unsafe {
            GetTokenInformation(
                self.handle,
                TokenIntegrityLevel,
                None,
                token_info_length,
                &mut token_info_length,
            )
        } {
            Err(e) if e.code() == HRESULT(0x8007007Au32 as i32) => (),
            Err(e) => return Err(FfiError::from_windows_error(e)),
            _ => unreachable!(),
        };

        let mut token_information = vec![0u8; token_info_length as usize];

        unsafe {
            GetTokenInformation(
                self.handle,
                TokenIntegrityLevel,
                Some(token_information.as_mut_ptr().cast()),
                token_information.len() as u32,
                &mut token_info_length,
            )
        }
        .map_err(FfiError::from_windows_error)?;

        Ok(SidAndAttributes(token_information))
    }
}

#[cfg(test)]
mod tests {
    use windows::Win32::Security::SID;

    use super::CurrentToken;

    #[test]
    fn test_integrity() {
        let token = CurrentToken::new();
        let integrity = token.integrity_level().unwrap();

        let sid = integrity.sid();

        let psid = sid.sid.as_ptr() as *const SID;
        unsafe { dbg!(*psid) };
    }
}
