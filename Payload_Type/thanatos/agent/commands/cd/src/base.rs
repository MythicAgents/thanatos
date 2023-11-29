/// Change the agent's directory
use agent_utils::{
    errors::ThanatosError,
    msg::{PendingTask, TaskResults},
};
use serde::Deserialize;

#[derive(Deserialize)]
struct CdArgs {
    directory: String,
}

pub fn cd(task: PendingTask) -> Result<TaskResults, ThanatosError> {
    let params: CdArgs =
        serde_json::from_str(&task.parameters).map_err(|_| ThanatosError::JsonDecodeError)?;

    std::env::set_current_dir(params.directory).map_err(|_| ThanatosError::os_error())?;

    Ok(TaskResults {
        completed: true,
        ..Default::default()
    })
}
