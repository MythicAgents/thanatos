use agent_utils::{
    errors::ThanatosError,
    msg::{ExtraInfoC2Profile, PendingTask, TaskResults},
};

use crate::agent::egress_agent::AgentShared;

pub fn profiles(
    _task: PendingTask,
    shared: &mut AgentShared,
) -> Result<TaskResults, ThanatosError> {
    let c2_profiles: Vec<ExtraInfoC2Profile> = shared
        .c2profiles
        .iter()
        .map(|profile| profile.borrow_into())
        .collect();

    Ok(TaskResults {
        completed: true,
        process_response: Some(
            serde_json::to_string(&c2_profiles).map_err(|_| ThanatosError::JsonEncodeError)?,
        ),
        ..Default::default()
    })
}
