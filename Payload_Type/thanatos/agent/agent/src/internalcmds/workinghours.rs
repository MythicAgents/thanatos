use agent_utils::{
    errors::ThanatosError,
    msg::{PendingTask, TaskResults},
};
use serde::Deserialize;

#[cfg(any(feature = "http", feature = "https"))]
use crate::agent::egress_agent::AgentShared;

#[cfg(feature = "tcp")]
use crate::agent::p2p_agent::AgentShared;

#[derive(Deserialize)]
struct WorkingHoursParameters {
    start: u64,
    end: u64,
}

/// Sets the working hours
pub fn workinghours(
    task: PendingTask,
    shared: &mut AgentShared,
) -> Result<TaskResults, ThanatosError> {
    let params: WorkingHoursParameters =
        serde_json::from_str(&task.parameters).map_err(|_| ThanatosError::JsonDecodeError)?;

    shared.working_start = std::time::Duration::from_secs(params.start);
    shared.working_end = std::time::Duration::from_secs(params.end);

    Ok(TaskResults {
        completed: true,
        ..Default::default()
    })
}
