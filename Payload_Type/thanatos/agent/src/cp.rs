use crate::agent::AgentTask;
use crate::mythic_success;
use crate::utils::unverbatim;
use serde::Deserialize;
use std::error::Error;
use std::path::Path;
use std::result::Result;

use path_clean::PathClean;

/// Struct containing the cp parameters
#[derive(Deserialize)]
struct CpArgs {
    /// Source path to copy
    source: String,

    /// Destination path to copy to
    destination: String,
}

/// Copies a file or directory from one location to another
/// * `task` - Task information from Mythic
pub fn copy_file(task: &AgentTask) -> Result<serde_json::Value, Box<dyn Error>> {
    // Parse the args into a struct
    let args: CpArgs = serde_json::from_str(&task.parameters)?;

    // Grab the source path from the arguments
    let s_path = Path::new(&args.source);

    // Check if the source path exists
    if !s_path.exists() {
        return Err(Box::new(std::io::Error::other(
            "Source path does not exist.",
        )));
    }

    // Convert the source path into an absolute path
    let s_path = std::env::current_dir()?.join(s_path).canonicalize()?;

    // Get the destination path
    let d_path = Path::new(&args.destination);

    // Check if the destination path exists and grab the absolute path.
    // If the path does not exist but is a directory, set the destination path to the
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

    // Copy the source path to the destination path
    std::fs::copy(&s_path, &d_path)?;

    let s_path = unverbatim(s_path).to_string_lossy().to_string();
    let d_path = unverbatim(d_path).to_string_lossy().to_string();

    // Return the output to Mythic
    Ok(mythic_success!(
        task.id,
        format!("Copied '{}' to '{}'", s_path, d_path)
    ))
}
