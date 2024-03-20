use std::{marker::PhantomData, ptr::NonNull};

use windows::{
    core::Error as WinError,
    Win32::{
        Foundation::{ERROR_INSUFFICIENT_BUFFER, HANDLE, PSID},
        Security::{
            GetSidSubAuthority, GetSidSubAuthorityCount, GetTokenInformation, TokenIntegrityLevel,
            SID_AND_ATTRIBUTES, TOKEN_MANDATORY_LABEL,
        },
        System::SystemServices::{
            SECURITY_MANDATORY_HIGH_RID, SECURITY_MANDATORY_LOW_RID, SECURITY_MANDATORY_MEDIUM_RID,
            SECURITY_MANDATORY_SYSTEM_RID,
        },
    },
};

use crate::errors::ThanatosError;

#[repr(u32)]
enum MythicIntegrityLevel {
    BelowLow = 0,
    Low = 1,
    Medium = 2,
    High = 3,
    System = 4,
}

impl Into<u32> for MythicIntegrityLevel {
    fn into(self) -> u32 {
        self as u32
    }
}

#[repr(transparent)]
struct CurrentToken(HANDLE);

impl CurrentToken {
    pub const fn new() -> CurrentToken {
        CurrentToken(HANDLE(-4))
    }

    pub fn integrity_level(&self) -> Result<TokenMandatoryLabel, WinError> {
        let mut info_length = 0u32;

        match unsafe {
            GetTokenInformation(
                self.0,
                TokenIntegrityLevel,
                None,
                info_length,
                &mut info_length,
            )
        } {
            Err(e) if e == ERROR_INSUFFICIENT_BUFFER.into() => (),
            Err(e) => return Err(e),
            _ => unreachable!(),
        }

        let mut token_buffer = vec![0u8; info_length as usize];

        unsafe {
            GetTokenInformation(
                self.0,
                TokenIntegrityLevel,
                Some(token_buffer.as_mut_ptr().cast()),
                info_length,
                &mut info_length,
            )?
        }

        Ok(TokenMandatoryLabel(token_buffer))
    }
}

struct TokenMandatoryLabel(Vec<u8>);

impl TokenMandatoryLabel {
    pub fn label(&mut self) -> SidAndAttributes {
        SidAndAttributes {
            ptr: unsafe {
                NonNull::new_unchecked(std::ptr::addr_of_mut!(
                    (*(self.0.as_mut_ptr().cast::<TOKEN_MANDATORY_LABEL>())).Label
                ))
            },
            _marker: PhantomData,
        }
    }
}

#[repr(transparent)]
struct SidAndAttributes<'a> {
    ptr: NonNull<SID_AND_ATTRIBUTES>,
    _marker: PhantomData<&'a SID_AND_ATTRIBUTES>,
}

impl<'a> SidAndAttributes<'a> {
    pub const fn sid(&self) -> Sid<'a> {
        Sid {
            ptr: unsafe { self.ptr.as_ref().Sid },
            _marker: PhantomData,
        }
    }
}

#[repr(transparent)]
struct Sid<'a> {
    ptr: PSID,
    _marker: PhantomData<&'a ()>,
}

impl<'a> Sid<'a> {
    pub fn sub_authority_count(&self) -> u8 {
        unsafe { *GetSidSubAuthorityCount(self.ptr) }
    }

    pub fn sub_authority(&self, index: u32) -> Option<u32> {
        let authority = unsafe { GetSidSubAuthority(self.ptr, index) };

        if authority.is_null() {
            None
        } else {
            Some(unsafe { *authority })
        }
    }
}

pub fn integrity_level() -> Result<u32, ThanatosError> {
    let token = CurrentToken::new();
    let mut token_integrity = token
        .integrity_level()
        .map_err(|e| ThanatosError::WinError(e.code()))?;

    let sid = token_integrity.label().sid();
    let level = sid
        .sub_authority((sid.sub_authority_count() - 1).into())
        .ok_or(ThanatosError::last_os_error())?;

    // Use range checks for the integrity level. Read https://devblogs.microsoft.com/oldnewthing/20221017-00/?p=10729
    // for reasoning
    if level >= SECURITY_MANDATORY_SYSTEM_RID as u32 {
        Ok(MythicIntegrityLevel::System.into())
    } else if level >= SECURITY_MANDATORY_HIGH_RID as u32 {
        Ok(MythicIntegrityLevel::High.into())
    } else if level >= SECURITY_MANDATORY_MEDIUM_RID as u32 {
        Ok(MythicIntegrityLevel::Medium.into())
    } else if level >= SECURITY_MANDATORY_LOW_RID as u32 {
        Ok(MythicIntegrityLevel::Low.into())
    } else {
        Ok(MythicIntegrityLevel::BelowLow.into())
    }
}
