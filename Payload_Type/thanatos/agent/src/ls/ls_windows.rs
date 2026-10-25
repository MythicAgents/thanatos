//! This file is only imported when compiling for Windows
use chrono::{DateTime, Local};
use serde::Serialize;
use std::os::windows::ffi::OsStrExt;
use std::{ffi::OsStr, iter::once, path};
use windows::Win32::Foundation::ERROR_INSUFFICIENT_BUFFER;
use windows::Win32::Security::Authorization::{
    ConvertStringSidToSidW, GetNamedSecurityInfoW, SE_FILE_OBJECT,
};
use windows::Win32::Security::{
    LookupAccountSidA, SidTypeUnknown, OWNER_SECURITY_INFORMATION, PSECURITY_DESCRIPTOR, PSID,
};
use windows_acl::acl::ACL;
use windows_strings::{PCWSTR, PSTR};

/// Struct holding the access control list for the list entry
#[derive(Debug, Default, Serialize)]
pub struct Acl {
    pub account: String,
    pub rights: String,
    pub r#type: String,
}

/// Struct holding the ACL permissions
#[derive(Default, Debug, Serialize)]
pub struct FilePermissions {
    acls: Vec<Acl>,
    creation_date: i64,
}

/// Get the ACLs for the specified path
/// * `fname` - File name for the object to get the ACLs
fn get_acls(fname: &str) -> Option<Vec<Acl>> {
    let mut acls: Vec<Acl> = Vec::<Acl>::new();
    let acl = ACL::from_file_path(fname, false).ok()?;
    let entries = acl.all().ok()?;
    for e in entries.iter() {
        let mut sid = PSID::default();
        let raw_string_sid = OsStr::new(e.string_sid.as_str())
            .encode_wide()
            .chain(once(0))
            .collect::<Vec<_>>();
        if unsafe { ConvertStringSidToSidW(PCWSTR(raw_string_sid.as_ptr()), &mut sid).is_ok() } {
            if let Some(account) = get_username_from_sid(sid) {
                if let Some(rights) = rights_mask_to_string(e.mask) {
                    acls.push(Acl {
                        account,
                        rights,
                        r#type: e.entry_type.to_string(),
                    });
                }
            }
        }
    }

    Some(acls)
}

/// Bit mask for extracting the file rights
const MASK_LIST: &[(u32, &str)] = &[(0x9, "Read"), (0x16, "Write"), (0x20, "Execute")];

/// Converts a file `ACCESS_MASK` to a String
/// * `mask` - Access mask to convert
fn rights_mask_to_string(mask: u32) -> Option<String> {
    // Check if the full control bits are set
    if 0x3f & mask == 0x3f {
        return Some("Full Control".to_string());
    }

    let mut rights = String::new();

    // Iterate over each bit mask entry and add the rights
    for (m_const, attr) in MASK_LIST {
        if m_const & mask != 0 {
            rights.push_str(format!("{}, ", attr).as_str());
        }
    }

    if !rights.is_empty() {
        Some(rights.trim_end_matches(", ").to_string())
    } else {
        None
    }
}

/// Converts a windows SID to a username using `LookupAccountSidA`
/// * `sid` - SID to get the username from
fn get_username_from_sid(sid: PSID) -> Option<String> {
    let mut dw_acct_name: u32 = 1;
    let mut dw_domain_name: u32 = 1;
    let mut e_use = SidTypeUnknown;

    // Initial call to `LookupAccountSidA` which will be used to get the right `dw_acct_name`
    // and `dw_domain_name` lengths
    if !unsafe {
        LookupAccountSidA(
            None,
            sid,
            None,
            &mut dw_acct_name,
            None,
            &mut dw_domain_name,
            &mut e_use,
        )
    }
    .is_err_and(|e| e.code() == ERROR_INSUFFICIENT_BUFFER.to_hresult())
    {
        return None;
    }

    // Create new buffers to hold the account and domain information
    let mut acct_name = vec![0; dw_acct_name as usize];
    let mut domain_name = vec![0; dw_domain_name as usize];

    // Grab the domain and owner attached to the file
    unsafe {
        LookupAccountSidA(
            None,
            sid,
            Some(PSTR(acct_name.as_mut_ptr().cast())),
            &mut dw_acct_name,
            Some(PSTR(domain_name.as_mut_ptr().cast())),
            &mut dw_domain_name,
            &mut e_use,
        )
        .ok()?;
    }

    // Check if the acount name was populated correctly
    if acct_name.is_empty() || acct_name[0] == 0 {
        return None;
    }

    // Remove the null-terminator from the account name and convert it to a String
    acct_name.pop()?;
    let account = std::str::from_utf8(acct_name.as_slice()).ok()?;

    // Check if the domain name was returned correctly
    if domain_name.is_empty() || domain_name[0] == 0 {
        // Domain name was not returned correctly so only return the account name of the file
        // owner
        Some(account.to_string())
    } else {
        // Domain name was returned correctly

        // Remove the null-terminator and convert it to a String
        domain_name.pop()?;
        let domain = std::str::from_utf8(domain_name.as_slice()).ok()?;

        // Return the domain and owner of the file
        Some(format!("{}\\{}", domain, account))
    }
}

/// Get the file owner name from a path
/// * `fname` - File path
pub fn get_file_owner(fname: &path::Path) -> Option<String> {
    // Canonicalize the path
    let fname = fname.canonicalize().ok()?;
    let mut fname = fname.into_os_string().encode_wide().collect::<Vec<_>>();
    fname.push(0);

    // Create a new SID object
    let mut psid_owner = PSID::default();

    let mut psd = PSECURITY_DESCRIPTOR::default();
    let psd_addr = &mut psd;

    // Grab the security information of the file
    unsafe {
        GetNamedSecurityInfoW(
            PCWSTR(fname.as_ptr()),
            SE_FILE_OBJECT,
            OWNER_SECURITY_INFORMATION,
            Some(&raw mut psid_owner),
            None,
            None,
            None,
            psd_addr,
        )
        .ok()
        .ok()?;
    }

    // Convert the SID to a username
    get_username_from_sid(psid_owner)
}

impl FilePermissions {
    /// Create a new `FilePermissions` object
    /// * `fpath` - Path to grab the permissions from
    pub fn new(fpath: &path::Path) -> Self {
        // Try to canonicalize the path
        let fpath = if let Ok(path) = fpath.canonicalize() {
            path
        } else {
            return Self {
                ..Default::default()
            };
        };

        // Null-terminate the file path
        let fname = if let Some(name) = fpath.to_str() {
            format!("{}\0", name)
        } else {
            return FilePermissions::default();
        };

        // Get the creation date timestamp
        let creation_date = fpath.metadata().ok().and_then(|meta| {
            meta.created().ok().and_then(|created| {
                (created >= std::time::UNIX_EPOCH)
                    .then(|| DateTime::<Local>::from(created).timestamp())
            })
        });

        // Return the file permissions
        FilePermissions {
            acls: get_acls(&fname).unwrap_or_else(|| vec![Default::default()]),
            creation_date: creation_date.unwrap_or_default(),
        }
    }
}
