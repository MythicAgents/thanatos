//! This file is only imported when compiling for Linux
use std::{
    os::unix::fs::{MetadataExt, PermissionsExt},
    path,
};

use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};

use crate::utils::{
    os::unix::{GroupEntry, PasswdEntry},
    parse_linux_mode,
};

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

        let permissions = meta.permissions();

        // Grab the uid and gid from the metadata
        let gid = meta.gid();
        let uid = meta.uid();

        // Lookup the group of the owner from the integer gid. Return just the integer gid
        // if that fails
        let group = GroupEntry::getgrgid(gid)
            .and_then(|entry| entry.ok_or_else(|| std::io::Error::from_raw_os_error(libc::ENOENT)))
            .map(|entry| entry.name().to_string_lossy().to_string())
            .unwrap_or_else(|_| gid.to_string());

        // Lookup the user from the integer uid. Return just the uid if that
        // fails
        let user = PasswdEntry::getpwuid(uid)
            .and_then(|entry| entry.ok_or_else(|| std::io::Error::from_raw_os_error(libc::ENOENT)))
            .map(|entry| entry.name().to_string_lossy().to_string())
            .unwrap_or_else(|_| uid.to_string());

        // Get the creation date timestamp
        let creation_date = meta.created().ok().and_then(|created| {
            (created >= std::time::UNIX_EPOCH).then(|| DateTime::<Local>::from(created).timestamp())
        });

        // Create a new FilePermissions object
        Self {
            uid: uid.try_into().unwrap_or(-1),
            gid: gid.try_into().unwrap_or(-1),
            permissions: parse_linux_mode(permissions.mode()),
            group,
            user,
            creation_date: creation_date.unwrap_or(-1),
        }
    }
}
