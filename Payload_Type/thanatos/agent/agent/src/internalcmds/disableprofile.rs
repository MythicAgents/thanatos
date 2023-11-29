use agent_utils::{
    errors::ThanatosError,
    msg::{PendingTask, TaskResults},
};
use serde::Deserialize;

use crate::agent::egress_agent::AgentShared;

#[derive(Deserialize)]
pub struct DisableProfileParameters {
    id: usize,
}

pub fn disable_profile(
    task: PendingTask,
    shared: &mut AgentShared,
) -> Result<TaskResults, ThanatosError> {
    let params: DisableProfileParameters =
        serde_json::from_str(&task.parameters).map_err(|_| ThanatosError::JsonDecodeError)?;

    // This check is handled server side so it should always succeed
    if let Some(profile) = shared
        .c2profiles
        .iter_mut()
        .find(|profile| profile.id == params.id)
    {
        profile.enabled = false;
    }

    Ok(TaskResults {
        completed: true,
        ..Default::default()
    })
}
