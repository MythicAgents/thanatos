use agent_utils::{
    errors::ThanatosError,
    msg::{PendingTask, TaskResults},
};
use serde::Deserialize;

use crate::agent::egress_agent::AgentShared;

/// Parameters for the `sleep` command
#[derive(Deserialize)]
struct SleepParameters {
    /// Sleep interval in milis
    interval: u64,

    /// Sleep jitter
    jitter: u32,
}

/// Sets the agent callback interval and jitter
pub fn sleep_agent(
    task: PendingTask,
    shared: &mut AgentShared,
) -> Result<TaskResults, ThanatosError> {
    let params: SleepParameters =
        serde_json::from_str(&task.parameters).map_err(|_| ThanatosError::JsonDecodeError)?;

    shared.callback_interval = std::time::Duration::from_secs(params.interval);
    shared.callback_jitter = params.jitter;

    Ok(TaskResults {
        completed: true,
        ..Default::default()
    })
}
