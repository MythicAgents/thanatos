//! This file is only imported when compiling for Linux
use crate::utils::{
    linux::{get_group_from_gid, get_user_from_uid},
    parse_linux_mode,
};
use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use std::convert::TryInto;
use std::path;

use std::os::linux::fs::MetadataExt;
use std::os::unix::fs::PermissionsExt;

/// Struct for storing file permissions
#[derive(Serialize, Deserialize, Default)]
pub struct FilePermissions {
    /// Username of the file owner
    pub user: String,

    /// Uid of the file owner
    pub uid: i32,

    /// Groupname of the file owner
    pub group: String,

    /// Gid of the file owner
    pub gid: i32,

    /// Posix permissions of the file
    pub permissions: String,

    /// Creation date of the file
    pub creation_date: i64,
}

/// Gets the file owner from a path
pub fn get_file_owner(file_path: &path::Path) -> String {
    // Get the file metadata
    let meta = if let Ok(data) = file_path.metadata() {
        data
    } else {
        return "".to_string();
    };

    // Get the owner uid
    let uid = meta.st_uid();

    // Convert the uid to a username
    get_user_from_uid(uid).unwrap_or_else(|| uid.to_string())
}

impl FilePermissions {
    /// Formulate a new `FilePermissions` object
    /// * `file_path` - File path to get the permissions from
    pub fn new(file_path: &path::Path) -> Self {
        // Grab the file metadata if there are no errors. Otherwise, populate a new FilePermissions
        // object with default information.
        let meta = if let Ok(data) = file_path.metadata() {
            data
        } else {
            return Self {
                ..Default::default()
            };
        };

        // Parse the integer mode into readable `-rwxrw-r--` text
        let permissions = parse_linux_mode(meta.permissions().mode());

        // Grab the uid and gid from the metadata
        let gid = meta.st_gid();
        let uid = meta.st_uid();

        // Lookup the group of the owner from the integer gid. Return just the integer gid
        // if that fails
        let group = get_group_from_gid(gid).unwrap_or_else(|| gid.to_string());

        // Lookup the user from the integer uid. Return just the uid if that
        // fails
        let user = get_user_from_uid(uid).unwrap_or_else(|| uid.to_string());

        // Get the creation date timestamp
        let creation_date = file_path
            .metadata()
            .ok()
            .and_then(|meta| {
                meta.created().ok().and_then(|created| {
                    (created >= std::time::UNIX_EPOCH)
                        .then(|| DateTime::<Local>::from(created).timestamp())
                })
            });

        // Create a new FilePermissions object
        Self {
            uid: uid.try_into().unwrap_or(-1),
            gid: gid.try_into().unwrap_or(-1),
            permissions,
            group,
            user,
            creation_date: creation_date.unwrap_or(0),
        }
    }
}
