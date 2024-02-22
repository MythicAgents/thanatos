use windows::Win32::System::Threading::{
    PROCESS_ACCESS_RIGHTS, PROCESS_ALL_ACCESS, PROCESS_CREATE_PROCESS, PROCESS_CREATE_THREAD,
    PROCESS_DUP_HANDLE, PROCESS_QUERY_INFORMATION, PROCESS_QUERY_LIMITED_INFORMATION,
};

use super::traits::*;

macro_rules! impl_access_rights {
    ($s:ident, $rights:ident) => {
        pub struct $s;
        impl $crate::internal::SealedTrait for $s {}
        impl ProcessAccessRights for $s {
            const RIGHTS: PROCESS_ACCESS_RIGHTS = $rights;
        }
    };
}

impl_access_rights!(ProcessAllAccess, PROCESS_ALL_ACCESS);
impl_access_rights!(ProcessCreateProcess, PROCESS_CREATE_PROCESS);
impl_access_rights!(ProcessCreateThread, PROCESS_CREATE_THREAD);
impl_access_rights!(ProcessDupHandle, PROCESS_DUP_HANDLE);
impl_access_rights!(ProcessQueryInformation, PROCESS_QUERY_INFORMATION);
impl_access_rights!(
    ProcessQueryLimitedInformation,
    PROCESS_QUERY_LIMITED_INFORMATION
);

impl ProcessQueryTokenRights for ProcessAllAccess {}
impl ProcessQueryTokenRights for ProcessQueryInformation {}
