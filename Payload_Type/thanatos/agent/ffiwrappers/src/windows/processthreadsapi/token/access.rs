use super::traits::*;
use windows::Win32::Security::{TOKEN_ACCESS_MASK, TOKEN_QUERY, TOKEN_QUERY_SOURCE};

macro_rules! impl_access_rights {
    ($s:ident, $rights:ident) => {
        pub struct $s;
        impl $crate::internal::SealedTrait for $s {}
        impl TokenAccessRights for $s {
            const MASK: windows::Win32::Security::TOKEN_ACCESS_MASK = $rights;
        }
    };
}

impl_access_rights!(TokenQuery, TOKEN_QUERY);
impl TokenQueryRights for TokenQuery {}
impl_access_rights!(TokenQuerySource, TOKEN_QUERY_SOURCE);
impl TokenQuerySourceRights for TokenQuerySource {}

pub struct TokenCurrent;
impl crate::internal::SealedTrait for TokenCurrent {}
impl TokenAccessRights for TokenCurrent {
    const MASK: TOKEN_ACCESS_MASK = TOKEN_ACCESS_MASK(TOKEN_QUERY.0 | TOKEN_QUERY_SOURCE.0);
}
impl TokenQueryRights for TokenCurrent {}
impl TokenQuerySourceRights for TokenCurrent {}
