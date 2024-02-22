pub mod access;

use crate::errors::FfiError;

use self::access::ProcessAllAccess;

use super::token::{traits::TokenAccessRights, CurrentToken, Token, TokenInner};
use std::marker::PhantomData;

use traits::*;
use windows::Win32::{
    Foundation::{CloseHandle, HANDLE},
    System::Threading::{OpenProcess, OpenProcessToken},
};

pub mod traits {
    use windows::Win32::System::Threading::PROCESS_ACCESS_RIGHTS;

    pub trait ProcessAccessRights: crate::internal::SealedTrait {
        const RIGHTS: PROCESS_ACCESS_RIGHTS;
    }

    pub trait ProcessQueryTokenRights: ProcessAccessRights {}
}

#[repr(transparent)]
struct ProcessInner<AccessRights: ProcessAccessRights> {
    handle: HANDLE,
    _access: PhantomData<AccessRights>,
}

#[repr(transparent)]
pub struct CurrentProcess(ProcessInner<access::ProcessAllAccess>);

impl CurrentProcess {
    pub const fn new() -> CurrentProcess {
        CurrentProcess(ProcessInner {
            handle: HANDLE(-1),
            _access: PhantomData,
        })
    }
    pub const fn token(&self) -> CurrentToken {
        CurrentToken::new()
    }
}

#[repr(transparent)]
pub struct Process<AccessRights: ProcessAccessRights>(ProcessInner<AccessRights>);

impl<AccessRights: ProcessAccessRights> Process<AccessRights> {
    pub fn open(pid: u32) -> Result<Process<AccessRights>, FfiError> {
        Ok(Self(ProcessInner {
            handle: unsafe {
                OpenProcess(AccessRights::RIGHTS, false, pid)
                    .map_err(FfiError::from_windows_error)?
            },
            _access: PhantomData,
        }))
    }
}

impl Process<ProcessAllAccess> {
    pub const fn current_process() -> CurrentProcess {
        CurrentProcess::new()
    }
}

impl<AccessRights: ProcessQueryTokenRights> Process<AccessRights> {
    pub fn token<TokenRights: TokenAccessRights>(&self) -> Result<Token<TokenRights>, FfiError> {
        let mut token_handle = HANDLE(0);

        unsafe { OpenProcessToken(self.0.handle, TokenRights::MASK, &mut token_handle) }
            .map_err(FfiError::from_windows_error)?;

        Ok(Token(TokenInner {
            handle: token_handle,
            _access: PhantomData,
        }))
    }
}

impl<AccessRights: ProcessAccessRights> Drop for Process<AccessRights> {
    fn drop(&mut self) {
        let _ = unsafe { CloseHandle(self.0.handle) };
    }
}
