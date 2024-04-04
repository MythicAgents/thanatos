//! This file is only imported when compiling for Windows
use chrono::{DateTime, Local};
use serde::Serialize;
use std::path;

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
    todo!();
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
        let creation_date = fpath
            .metadata()
            .ok()
            .map(|meta| {
                meta.created().ok().and_then(|created| {
                    (created >= std::time::UNIX_EPOCH)
                        .then(|| DateTime::<Local>::from(created).timestamp())
                })
            })
            .flatten();

        // Return the file permissions
        FilePermissions {
            acls: get_acls(&fname).unwrap_or_else(|| vec![Default::default()]),
            creation_date: creation_date.unwrap_or_default(),
        }
    }
}
