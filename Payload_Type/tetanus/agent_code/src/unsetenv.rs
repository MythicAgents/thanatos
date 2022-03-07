use crate::agent::AgentTask;
use crate::mythic_success;
use serde::Deserialize;
use std::error::Error;
use std::result::Result;

/// Struct for holding the unsetenv parameters
#[derive(Deserialize)]
struct UnsetenvArgs {
    /// Environment variable to unset
    variable: String,
}

/// Unsets an environment variable
pub fn unset_env(task: &AgentTask) -> Result<serde_json::Value, Box<dyn Error>> {
    let args: UnsetenvArgs = serde_json::from_str(&task.parameters)?;

    // Unset the variable
    std::env::remove_var(&args.variable);

    // Send the completed task information to Mythic
    Ok(mythic_success!(
        task.id,
        format!("Removed environment variable '{}'", args.variable)
    ))
}
