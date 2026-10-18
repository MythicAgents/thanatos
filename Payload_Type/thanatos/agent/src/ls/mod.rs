use crate::agent::AgentTask;
use crate::utils::unverbatim;
use chrono::prelude::{DateTime, Local};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::error::Error;
use std::path::Path;

// Import dependencies based on platform configuration
cfg_if::cfg_if! {
    if #[cfg(target_os = "linux")] {
        pub mod ls_linux;
        use ls_linux::{FilePermissions, get_file_owner};
        use crate::utils::linux as native;
    } else if #[cfg(target_os = "windows")] {
        mod ls_windows;
        use ls_windows::{FilePermissions, get_file_owner};
        use crate::utils::windows as native;
    }
}

/// Parameters for the ls command
#[allow(dead_code)]
#[derive(Deserialize)]
struct LsArgs {
    /// Host to get a listing from
    host: String,

    /// Path to list
    path: String,

    /// File in the path to list
    file: String,

    /// If the task was from the file browser
    file_browser: Option<bool>,
}

/// Struct for storing the Mythic info needed for the file browser
#[derive(Serialize)]
pub struct FileBrowser {
    /// Host that the listing if from
    pub host: String,

    /// Platform that the listing is from (Windows, Linux, ssh)
    pub platform: String,

    /// If the listing entry is a file
    pub is_file: bool,

    /// Permissions of the list entry
    pub permissions: FilePermissions,

    /// Name of the file/directory being listed
    pub name: String,

    /// Parent path of the listing entry
    pub parent_path: String,

    /// If the listing was successful
    pub success: bool,

    /// Access time for the list entry
    pub access_time: i64,

    /// Modify time for the list entry
    pub modify_time: i64,

    /// Size of the list entry
    pub size: u64,

    /// Flag signifying if the Mythic file browser should update delete files
    pub update_deleted: bool,

    /// List of children for this parent listing
    pub files: Vec<File>,
}

/// Struct for holding information about each child path from a listing
#[derive(Serialize)]
pub struct File {
    /// If the object being listed is a file
    pub is_file: bool,

    /// Permissions of the listing entry
    pub permissions: FilePermissions,

    /// Name of the listing entry
    pub name: String,

    /// Full name of the listing entry
    pub full_name: String,

    /// Access time of the listing entry
    pub access_time: i64,

    /// Modify time of the listing entry
    pub modify_time: i64,

    /// Size of the listing entry
    pub size: u64,

    /// Owner of the listing entry
    pub owner: String,
}

impl FileBrowser {
    /// Generate a new `FileBrowser` listing
    /// * `args` - Argumuments for the Mythic command
    fn new(args: &LsArgs) -> Result<Self, Box<dyn Error>> {
        // Get the current working directory
        let cwd = std::env::current_dir()?;

        // Extract the path from the parameters
        let path = Path::new(&args.path);

        // Find the absolute path of the requested file listing
        let path = Path::new(&cwd.join(path)).canonicalize()?;

        // Get the last item in the path
        let mut name = String::from(
            path.components()
                .next_back()
                .ok_or("")?
                .as_os_str()
                .to_string_lossy(),
        );

        // If listing the root directory on windows, fix the name
        if name == "\\" {
            name = path
                .components()
                .next()
                .ok_or("")?
                .as_os_str()
                .to_string_lossy()
                .trim_start_matches(r"\\?\")
                .to_string();

            name.push('\\');
        }

        // Grab the parent path if it exists
        let parent_path = unverbatim(if let Some(parent) = path.parent() {
            parent.to_path_buf()
        } else {
            std::path::PathBuf::from("")
        })
        .as_os_str()
        .to_string_lossy()
        .to_string();

        let mut files: Vec<File> = Vec::new();

        // Iterate over each entry in the directory
        let dir_entries = path.read_dir()?;
        for entry in dir_entries.flatten() {
            // Append the entry information if it is readable
            if let Ok(file) = File::new(&entry.path()) {
                files.push(file);
            }
        }

        let mut access_time = 0i64;
        let mut modify_time = 0i64;

        let mut size = 0;

        // Get the access time, modify time and creation date from the path metadata
        if let Ok(meta) = path.metadata() {
            if let Ok(accessed) = meta.accessed() {
                if accessed >= std::time::UNIX_EPOCH {
                    access_time = DateTime::<Local>::from(accessed).timestamp_millis();
                }
            }

            if let Ok(modified) = meta.modified() {
                if modified >= std::time::UNIX_EPOCH {
                    modify_time = DateTime::<Local>::from(modified).timestamp_millis();
                }
            }

            size = meta.len()
        }

        // Create the file browser object
        Ok(Self {
            host: args.host.to_string(),
            platform: native::whoami::generic_platform(),
            is_file: path.is_file(),
            permissions: FilePermissions::new(&path),
            name,
            parent_path,
            success: true,
            access_time,
            modify_time,
            size,
            update_deleted: true,
            files,
        })
    }
}

impl File {
    /// Creates a new `File` struct
    /// * `path` - Path to the file
    fn new(path: &Path) -> Result<Self, Box<dyn Error>> {
        let path = std::path::Path::new(path);

        // Grab the absolute path to the file
        let full_name = unverbatim(path_clean::clean(&path.to_string_lossy()).into());

        // Get the name of the file
        let name = if full_name.is_file() {
            if let Some(name) = path.file_name() {
                name.to_string_lossy()
            } else {
                std::ffi::OsStr::new("").to_string_lossy()
            }
        } else {
            path.components()
                .next_back()
                .ok_or("")?
                .as_os_str()
                .to_string_lossy()
        }
        .to_string();

        let full_name = full_name.to_string_lossy().to_string();

        // Grab the time metadata from the file
        let mut access_time = 0i64;
        let mut modify_time = 0i64;

        let mut size = 0;

        if let Ok(path) = path.canonicalize() {
            if let Ok(meta) = path.metadata() {
                if let Ok(accessed) = meta.accessed() {
                    if accessed >= std::time::UNIX_EPOCH {
                        access_time = DateTime::<Local>::from(accessed).timestamp_millis();
                    }
                }

                if let Ok(modified) = meta.modified() {
                    if modified >= std::time::UNIX_EPOCH {
                        modify_time = DateTime::<Local>::from(modified).timestamp_millis();
                    }
                }

                size = meta.len()
            }
        }

        // Create the File struct
        Ok(Self {
            is_file: path.is_file(),
            permissions: FilePermissions::new(path), // Get the permissions of the file
            name,
            full_name,
            access_time,
            modify_time,
            size,
            owner: get_file_owner(path),
        })
    }
}

/// Get a directory listing of a path
/// * `task` - Task information
pub fn make_ls(task: &AgentTask) -> Result<serde_json::Value, Box<dyn Error>> {
    // Parse the arguments into a parsable object
    let args: LsArgs = serde_json::from_str(&task.parameters)?;

    // Formulate a new file_browser object
    let file_browser = FileBrowser::new(&args)?;

    // Return the data to Mythic
    Ok(json!({
        "task_id": task.id,
        "file_browser": &file_browser,
        "status": "success",
        "completed": true,
        "user_output": serde_json::to_string(&file_browser)?,
    }))
}
