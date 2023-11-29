use agent_utils::{
    errors::ThanatosError,
    msg::{PendingTask, TaskResults},
};

impl crate::Tasker {
    pub fn run_loaded(&mut self, _task: PendingTask) -> Result<Option<TaskResults>, ThanatosError> {
        todo!();
    }
}
