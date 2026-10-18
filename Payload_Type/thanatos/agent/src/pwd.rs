use crate::agent::AgentTask;
use crate::mythic_success;
use crate::utils::unverbatim;
use std::env;
use std::error::Error;
use std::result::Result;

/// Prints the current working directory
/// * `task` - Task information
pub fn get_pwd(task: &AgentTask) -> Result<serde_json::Value, Box<dyn Error>> {
    // Grab the current working directory
    let pwd = unverbatim(env::current_dir()?)
        .to_string_lossy()
        .to_string();

    // Return the cwd to Mythic
    Ok(mythic_success!(task.id, pwd))
}
