//! This file is only imported when compiling for Windows
use serde::Serialize;
use std::os::windows::ffi::OsStrExt;
use std::{ffi::OsStr, iter::once, path, ptr};
use winapi::shared::ntdef::NULL;
use winapi::shared::sddl::ConvertStringSidToSidW;
use winapi::um::{
    accctrl::SE_FILE_OBJECT,
    aclapi::GetNamedSecurityInfoA,
    errhandlingapi::GetLastError,
    winbase::LookupAccountSidA,
    winnt::{SidTypeUnknown, ACCESS_MASK, OWNER_SECURITY_INFORMATION, PSECURITY_DESCRIPTOR, PSID},
};
use windows_acl::acl::ACL;

use winapi::ctypes::c_void;

/// Struct holding the access control list for the list entry
#[derive(Debug, Default, Serialize)]
pub struct Acl {
    pub account: String,
    pub rights: String,
    pub r#type: String,
}

/// Struct holding the ACL permissions
#[derive(Default, Debug, Serialize)]
pub struct FilePermissions(Vec<Acl>);

/// Get the ACLs for the specified path
/// * `fname` - File name for the object to get the ACLs
fn get_acls(fname: &str) -> Option<Vec<Acl>> {
    let mut acls: Vec<Acl> = Vec::<Acl>::new();
    let acl = ACL::from_file_path(fname, false).ok()?;
    let entries = acl.all().ok()?;
    for e in entries.iter() {
        let mut sid: PSID = NULL as PSID;
        let raw_string_sid: Vec<u16> = OsStr::new(e.string_sid.as_str())
            .encode_wide()
            .chain(once(0))
            .collect();
        if unsafe { ConvertStringSidToSidW(raw_string_sid.as_ptr(), &mut sid) } != 0 {
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
fn rights_mask_to_string(mask: ACCESS_MASK) -> Option<String> {
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
    unsafe {
        LookupAccountSidA(
            ptr::null_mut(),
            sid as PSID,
            ptr::null_mut(),
            &mut dw_acct_name,
            ptr::null_mut(),
            &mut dw_domain_name,
            &mut e_use,
        )
    };

    if unsafe { GetLastError() } != 122 {
        return None;
    }

    // Create new buffers to hold the account and domain information
    let mut acct_name: Vec<u8> = Vec::new();
    acct_name.resize(dw_acct_name as usize, 0);

    let mut domain_name: Vec<u8> = Vec::new();
    domain_name.resize(dw_domain_name as usize, 0);

    // Grab the domain and owner attached to the file
    if unsafe {
        LookupAccountSidA(
            ptr::null_mut(),
            sid as PSID,
            acct_name.as_ptr() as *mut i8,
            &mut dw_acct_name,
            domain_name.as_ptr() as *mut i8,
            &mut dw_domain_name,
            &mut e_use,
        )
    } == 0
    {
        return None;
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
pub fn get_file_owner(fname: &path::Path) -> String {
    // Canonicalize the path
    let fname = if let Ok(fname) = fname.canonicalize() {
        if let Some(name) = fname.to_str() {
            format!("{}\0", name)
        } else {
            return "".to_string();
        }
    } else {
        return "".to_string();
    };

    // Create a new SID object
    let mut psid_owner: PSID = ptr::null_mut();
    let psid_owner_addr: *mut *mut c_void = &mut psid_owner;

    let mut psd: PSECURITY_DESCRIPTOR = ptr::null_mut();
    let psd_addr: *mut *mut c_void = &mut psd;

    // Grab the security information of the file
    if unsafe {
        GetNamedSecurityInfoA(
            fname.as_ptr() as *const i8,
            SE_FILE_OBJECT,
            OWNER_SECURITY_INFORMATION,
            psid_owner_addr,
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null_mut(),
            psd_addr,
        )
    } != 0
    {
        return "".to_string();
    }

    // Convert the SID to a username
    get_username_from_sid(psid_owner).unwrap_or_else(|| "".to_string())
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
            return FilePermissions(Vec::new());
        };

        // Return the file permissions
        FilePermissions(get_acls(&fname).unwrap_or_else(|| vec![Default::default()]))
    }
}
