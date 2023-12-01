use crate::agent::AgentTask;
use crate::mythic_success;
use serde::Deserialize;
use std::error::Error;
use std::result::Result;
use std::str::FromStr;

/// Struct containing the sleep parameters
#[derive(Deserialize)]
struct SleepArgs {
    /// Interval in seconds to sleep
    interval: u64,

    /// Jitter in percentage from 0-100
    jitter: u64,
}

/// Sets the agent's sleep interval and jitter
/// * `task` - Task information
/// * `agent_interval` - Currently stored agent sleep interval
/// * `agent_jitter` - Currently stored agent sleep jitter
pub fn set_sleep(
    task: &AgentTask,
    agent_interval: &mut u64,
    agent_jitter: &mut u64,
) -> Result<serde_json::Value, Box<dyn Error>> {
    // Parse the task arguments
    let args: SleepArgs = serde_json::from_str(&task.parameters).unwrap();

    // Set the new interval and jitter
    *agent_interval = args.interval;
    *agent_jitter = args.jitter;

    // Formulate the task output
    let output = format!(
        "Set new sleep interval to {} second(s) with a jitter of {}%",
        args.interval, args.jitter
    );

    // Send the output up to Mythic
    Ok(mythic_success!(task.id, output))
}
