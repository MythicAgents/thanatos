use crate::agent::AgentTask;
use crate::mythic_success;
use serde::Deserialize;
use std::fs;
use std::io::Result;
use std::path::Path;

/// Struct for the `mkdir` command parameters
#[derive(Deserialize)]
struct MkdirArgs {
    /// Directory to create
    directory: String,
}

pub fn make_directory(task: &AgentTask) -> Result<serde_json::Value> {
    // Parse the task arguments
    let args: MkdirArgs = serde_json::from_str(&task.parameters)?;

    // Create a new directory
    fs::create_dir_all(&args.directory)?;

    // Formulate the absolute path of the newly created directory
    let dir = Path::new(&args.directory);
    let dir = if dir.is_absolute() {
        String::from(dir.canonicalize()?.to_string_lossy())
    } else {
        let dir = std::env::current_dir()?.join(&dir);
        String::from(dir.canonicalize()?.to_string_lossy())
    };

    // Return the path to the new directory
    Ok(mythic_success!(
        task.id,
        format!("Created directory '{}'", dir)
    ))
}
