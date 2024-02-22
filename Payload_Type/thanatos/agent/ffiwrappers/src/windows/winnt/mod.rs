use std::{marker::PhantomData, ptr::NonNull};

use windows::Win32::Security::{SID, SID_AND_ATTRIBUTES};

pub struct SidAndAttributes(pub(crate) Vec<u8>);

impl SidAndAttributes {
    pub fn sid(&self) -> SidRef<'_> {
        let sid_and_attributes: *const SID_AND_ATTRIBUTES = self.0.as_ptr().cast();

        SidRef {
            sid: unsafe {
                NonNull::new((*sid_and_attributes).Sid.0 as *mut SID).unwrap_unchecked()
            },
            _marker: PhantomData,
        }
    }
}

#[repr(transparent)]
pub struct SidRef<'a> {
    pub(crate) sid: NonNull<SID>,
    _marker: PhantomData<&'a SID>,
}

impl SidRef<'_> {
    pub const fn identifier_authority(&self) -> [u8; 6] {
        unsafe { self.sid.as_ref().IdentifierAuthority.Value }
    }

    pub const fn subauthorities(&self) -> &[u32] {
        let subauthority_count = unsafe { self.sid.as_ref().SubAuthorityCount } as usize;
        unsafe {
            std::slice::from_raw_parts(self.sid.as_ref().SubAuthority.as_ptr(), subauthority_count)
        }
    }
}
