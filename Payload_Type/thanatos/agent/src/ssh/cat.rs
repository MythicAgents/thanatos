use ssh2::Session;
use std::error::Error;
use std::io::Read;
use std::path::Path;
use std::result::Result;

use crate::agent::AgentTask;
use crate::mythic_success;

use super::SshArgs;

/// Read a file from a remote server using SSH
/// * `sess` - Connected SSH session
/// * `task` - Mythic task used to invoke the command
/// * `args` - Arguments for the task
pub fn ssh_cat(
    sess: Session,
    task: &AgentTask,
    args: &SshArgs,
) -> Result<serde_json::Value, Box<dyn Error>> {
    // Get the file name from the arguments
    let file_name = args.cat.as_ref().unwrap();

    let file = Path::new(file_name);

    // Try to open the remote file
    let (mut rem_file, stat) = sess.scp_recv(file)?;

    // Read the file contents
    let mut file_contents: Vec<u8> = Vec::with_capacity(stat.size() as usize);
    rem_file.read_to_end(&mut file_contents)?;

    // Close the SSH session
    rem_file.send_eof()?;
    rem_file.wait_eof()?;
    rem_file.close()?;
    rem_file.wait_close()?;

    // Formulate the output for sending to the Mythic UI
    let mut output = mythic_success!(task.id, std::str::from_utf8(&file_contents)?);
    let output = output.as_object_mut().unwrap();
    output.insert(
        "artifacts".to_string(),
        serde_json::json!([
            {
                "base_artifact": "Remote FileOpen",
                "artifact": format!("ssh {}@{} -cat {}", args.credentials.account, args.host, file_name)
            }
        ])
    );

    Ok(serde_json::to_value(output)?)
}
