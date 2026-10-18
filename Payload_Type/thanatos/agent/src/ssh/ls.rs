use crate::utils::parse_linux_mode;
use serde::{Deserialize, Serialize};
use serde_json::json;
use ssh2::Session;
use std::error::Error;
use std::path::Path;
use std::path::PathBuf;
use std::result::Result;

/// Struct for holding file information
#[derive(Serialize)]
struct File {
    /// The entry is a file
    is_file: bool,

    /// Permissions of the entry
    permissions: FilePermissions,

    /// Name of the entry
    name: String,

    /// Full path to the entry
    full_name: String,

    /// Last access time of the entry
    access_time: u64,

    /// Last modify time of the entry
    modify_time: u64,

    /// Size of the entry
    size: u64,
}

/// Struct for holding file permissions
#[derive(Serialize, Deserialize)]
struct FilePermissions {
    /// UID owner of the entry
    uid: u32,

    /// GID owner of the entry
    gid: u32,

    /// Permissions for the entry
    permissions: String,
}

/// Struct for holding the file browser information
#[derive(Serialize)]
struct FileBrowser {
    /// Host the file entries are from
    host: String,

    /// Platform for the browser script (populates to "ssh")
    platform: String,

    /// The entry is a file
    is_file: bool,

    /// Permissions for the entry
    permissions: FilePermissions,

    /// Name of the entry
    name: String,

    /// Parent path of the entry
    parent_path: String,

    /// If the file browse was a success
    success: bool,

    /// Last access time for the entry
    access_time: u64,

    /// Last modify time for the entry
    modify_time: u64,

    /// Size of the entry
    size: u64,

    /// Whether to update delete file browse entries
    update_deleted: bool,

    /// List of the children for the ssh-ls entry
    files: Vec<File>,
}

impl FilePermissions {
    /// Create a new `FilePermissions` struct from file stat information
    /// * `fstat` - File stat information
    fn new(fstat: &ssh2::FileStat) -> Self {
        // Get the unix permissions
        let permissions = parse_linux_mode(fstat.perm.unwrap());

        // Get the uid
        let uid = fstat.uid.unwrap();

        // Get the gid
        let gid = fstat.gid.unwrap();

        Self {
            uid,
            gid,
            permissions,
        }
    }
}

impl File {
    /// Create a new `File` entry
    /// * `stats` - Tuple of both the path to the file and the stat information
    fn new(stats: (PathBuf, ssh2::FileStat)) -> Result<Self, Box<dyn Error>> {
        // Destructure the path and stat information
        let (path, stats) = stats;

        let path = Path::new(&path);

        // Get the full path to the file
        let full_name = path_clean::clean(&path.to_string_lossy());

        // Get the name of the file
        let name = if stats.is_file() {
            String::from(path.file_name().unwrap().to_string_lossy())
        } else {
            String::from(
                path.components()
                    .next_back()
                    .ok_or("")?
                    .as_os_str()
                    .to_string_lossy(),
            )
        };

        let access_time = stats.atime.map(|access_ts| access_ts * 1000).unwrap_or(0);
        let modify_time = stats.mtime.map(|modify_ts| modify_ts * 1000).unwrap_or(0);

        Ok(Self {
            is_file: stats.is_file(),
            permissions: FilePermissions::new(&stats),
            name,
            full_name,
            access_time,
            modify_time,
            size: stats.size.unwrap(),
        })
    }
}

impl FileBrowser {
    /// Create a new `FileBrowser` struct for `ssh-ls`
    /// * `sess` - Connected SSH session
    /// * `path` - Path for the new ssh-ls entry
    /// * `host` - Host this entry is from
    fn new(sess: Session, path: &str, host: String) -> Result<Self, Box<dyn Error>> {
        // Grab the last item in the file path
        let path = Path::new(&path);
        let mut name = String::from(
            path.components()
                .next_back()
                .ok_or("")?
                .as_os_str()
                .to_string_lossy(),
        );

        // Convert the root dir to a forward slash if needed
        if name == "\\" {
            name = "/".to_string();
        }

        // Grab the parent path if it exists
        let parent_path = if let Some(parent) = path.parent() {
            String::from(parent.to_string_lossy())
        } else {
            "".to_string()
        };

        // Setup a new sftp session
        let sftp_sess = sess.sftp()?;

        let mut sftp_path = sftp_sess.open(path)?;
        let path_stat = sftp_path.stat()?;

        // Iterate over each file in the path
        let mut files: Vec<File> = Vec::new();
        for entry in sftp_sess.readdir(path)? {
            if let Ok(entry) = File::new(entry) {
                files.push(entry);
            }
        }

        let access_time = path_stat
            .atime
            .map(|access_ts| access_ts * 1000)
            .unwrap_or(0);

        let modify_time = path_stat
            .mtime
            .map(|modify_ts| modify_ts * 1000)
            .unwrap_or(0);

        Ok(Self {
            host,
            platform: "ssh".to_string(), // platform is ssh, used for browser script
            is_file: path_stat.is_file(),
            permissions: FilePermissions::new(&path_stat),
            name,
            parent_path,
            success: true,
            access_time,
            modify_time,
            size: path_stat.size.unwrap(),
            update_deleted: true,
            files,
        })
    }
}

/// Entrypoint for the `ssh -ls` command
/// * `sess` - Connected SSH session
/// * `path` - Path for the `ssh -ls` entry
/// * `task_id` - Mythic task Id
/// * `host` - Host the entry is from
pub fn ssh_list(
    sess: Session,
    path: &str,
    task_id: &str,
    host: String,
) -> Result<serde_json::Value, Box<dyn Error>> {
    // Grab the file browser data
    let file_browser = FileBrowser::new(sess, path, host)?;

    // Send the response to Mythic
    Ok(json!({
        "task_id": task_id,
        "file_browser": &file_browser,
        "status": "success",
        "completed": true,
        "user_output": serde_json::to_string(&file_browser)?,
    }))
}
