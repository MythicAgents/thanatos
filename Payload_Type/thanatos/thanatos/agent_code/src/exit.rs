use crate::agent::AgentTask;
use crate::mythic_success;

/// Exits the agent
/// * `task` - Tasking information
/// * `agent_exit` - Flag used for signifying if the agent should exit
pub fn exit_agent(task: &AgentTask, agent_exit: &mut bool) -> serde_json::Value {
    *agent_exit = true;
    mythic_success!(task.id, "Exiting...")
}
