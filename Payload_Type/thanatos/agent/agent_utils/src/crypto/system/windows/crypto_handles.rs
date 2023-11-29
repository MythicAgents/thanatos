use crate::ThanatosError;
use std::os::windows::prelude::{AsRawHandle, BorrowedHandle, RawHandle};
use windows::Win32::{
    Foundation::HANDLE,
    Security::Cryptography::{
        BCryptCloseAlgorithmProvider, BCryptDestroyHash, BCryptDestroyKey, BCRYPT_ALG_HANDLE,
        BCRYPT_HASH_HANDLE, BCRYPT_KEY_HANDLE,
    },
};

#[repr(transparent)]
pub struct OwnedAlgorithmProviderHandle {
    handle: RawHandle,
}

impl OwnedAlgorithmProviderHandle {
    #[inline]
    pub fn as_handle(&self) -> BorrowedHandle<'_> {
        unsafe { BorrowedHandle::borrow_raw(self.as_raw_handle()) }
    }
}

impl Drop for OwnedAlgorithmProviderHandle {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            // | https://learn.microsoft.com/en-us/windows/win32/api/bcrypt/nf-bcrypt-bcryptclosealgorithmprovider
            // | [in] dwFlags
            // |
            // | A set of flags that modify the behavior of this function. "No flags are defined for this function."
            //
            // Nice, a function that closes a handle which takes in flags that aren't defined. Windows is great....
            let _ = BCryptCloseAlgorithmProvider(BCRYPT_ALG_HANDLE(self.handle as _), 0);
        }
    }
}

impl TryFrom<isize> for OwnedAlgorithmProviderHandle {
    type Error = ThanatosError;

    #[inline]
    fn try_from(handle: isize) -> Result<Self, Self::Error> {
        if HANDLE(handle).is_invalid() {
            Err(ThanatosError::InvalidHandle)
        } else {
            Ok(Self {
                handle: handle as _,
            })
        }
    }
}

impl AsRawHandle for OwnedAlgorithmProviderHandle {
    #[inline]
    fn as_raw_handle(&self) -> RawHandle {
        self.handle
    }
}

#[repr(transparent)]
pub struct OwnedBCryptKeyHandle {
    handle: RawHandle,
}

impl OwnedBCryptKeyHandle {
    #[inline]
    pub fn as_handle(&self) -> BorrowedHandle<'_> {
        unsafe { BorrowedHandle::borrow_raw(self.as_raw_handle()) }
    }
}

impl Drop for OwnedBCryptKeyHandle {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            let _ = BCryptDestroyKey(BCRYPT_KEY_HANDLE(self.handle as _));
        }
    }
}

impl TryFrom<isize> for OwnedBCryptKeyHandle {
    type Error = ThanatosError;

    #[inline]
    fn try_from(handle: isize) -> Result<Self, ThanatosError> {
        if HANDLE(handle).is_invalid() {
            Err(ThanatosError::InvalidHandle)
        } else {
            Ok(Self {
                handle: handle as _,
            })
        }
    }
}

impl AsRawHandle for OwnedBCryptKeyHandle {
    #[inline]
    fn as_raw_handle(&self) -> RawHandle {
        self.handle
    }
}

#[repr(transparent)]
pub struct OwnedBCryptHashHandle {
    handle: RawHandle,
}

impl OwnedBCryptHashHandle {
    #[inline]
    pub fn as_handle(&self) -> BorrowedHandle<'_> {
        unsafe { BorrowedHandle::borrow_raw(self.as_raw_handle()) }
    }
}

impl Drop for OwnedBCryptHashHandle {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            let _ = BCryptDestroyHash(BCRYPT_HASH_HANDLE(self.handle as _));
        }
    }
}

impl TryFrom<isize> for OwnedBCryptHashHandle {
    type Error = ThanatosError;

    #[inline]
    fn try_from(handle: isize) -> Result<Self, ThanatosError> {
        if HANDLE(handle).is_invalid() {
            Err(ThanatosError::InvalidHandle)
        } else {
            Ok(Self {
                handle: handle as _,
            })
        }
    }
}

impl AsRawHandle for OwnedBCryptHashHandle {
    #[inline]
    fn as_raw_handle(&self) -> RawHandle {
        self.handle
    }
}
