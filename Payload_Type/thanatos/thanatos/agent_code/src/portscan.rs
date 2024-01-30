use crate::agent::AgentTask;
use crate::{mythic_continued, mythic_success};
use serde::Deserialize;
use std::error::Error;
use std::io::ErrorKind;
use std::net::{Ipv4Addr, SocketAddr, TcpStream};
use std::result::Result;
use std::str::FromStr;
use std::sync::mpsc;
use std::time::Duration;

/// Struct containing the parameters for `portscan`
#[derive(Deserialize)]
struct PortscanArgs {
    /// List of hosts to scan
    hosts: Vec<String>,

    /// Comma/hyphen separated list of ports to scan
    ports: String,

    /// Sleep interval for TCP connections
    interval: u64,
}

/// Scans a list of IPs and ports with a delay
/// * `tx` - Channel for sending information to Mythic
/// * `rx` - Channel for receiving information from Mythic
pub fn scan_ports(
    tx: &mpsc::Sender<serde_json::Value>,
    rx: mpsc::Receiver<serde_json::Value>,
) -> Result<(), Box<dyn Error>> {
    // Parse the initial task information
    let task: AgentTask = serde_json::from_value(rx.recv()?)?;
    let args: PortscanArgs = serde_json::from_str(&task.parameters)?;

    // Split the comma-separated list of ports
    let port_split: Vec<&str> = args.ports.split(',').collect();

    // Create the full list of ports including ranges in integer format
    let mut ports: Vec<u16> = Vec::new();

    // Iterate over each port/port range
    for port in port_split {
        // Check if this is a port range
        if port.contains('-') {
            // Get the start and end for the range
            let range: Vec<u16> = port.split('-').map(|p| p.parse::<u16>().unwrap()).collect();
            let start = range[0];
            let end = range[1];

            // Add the range of ports to the port list
            ports.append(&mut (start..=end).map(u16::from).collect());

            continue;
        }

        // Add the port to the list if it is not a range
        ports.push(port.parse::<u16>()?);
    }

    // Create the list of hosts to scan
    let mut hosts: Vec<Ipv4Addr> = Vec::new();

    // Iterate over the list of hosts/subnets
    for host in args.hosts {
        // Check if the entry is a subnet and parse it
        if host.contains('/') {
            hosts.append(&mut parse_subnet(host)?);
        } else {
            hosts.push(Ipv4Addr::from_str(&host)?);
        }
    }

    // Notify the operator that the port scan is starting
    tx.send(mythic_continued!(
        task.id,
        "scanning",
        "Scanning hosts...\n"
    ))?;

    // Iterate over each host to scan
    for host in hosts.iter() {
        // Iterate over each port to scan for the host
        for port in ports.iter() {
            // Try to connect to the port
            let connection = format!("{}:{}", host, port);
            let connection = SocketAddr::from_str(&connection)?;

            // If the connection was successful, relay the output to the operator at the next
            // callback instead of waiting until the port scan completes
            if TcpStream::connect_timeout(&connection, Duration::from_secs(1)).is_ok() {
                tx.send(mythic_continued!(
                    task.id,
                    "scanning",
                    format!("Found open port at {}\n", &connection)
                ))?;
            }

            // Check if the channel to send the task information is still open.
            // If the channel is disconnected, this means that the operator killed the job.
            // The check is here so that the port scan will stop when the operator wants
            // it to instead of continuing to scan even after the job is killed
            if let Err(e) = rx.try_recv() {
                if e == std::sync::mpsc::TryRecvError::Disconnected {
                    return Ok(());
                }
            }

            // Sleep the specified miliseconds
            std::thread::sleep(Duration::from_millis(args.interval));
        }
    }

    // Notify Mythic that the portscan is complete
    tx.send(mythic_success!(task.id, "\nFinished portscan"))?;
    Ok(())
}

/// Parses a subnet string into a list of IP addresses
/// * `subnet` - Subnet to parse
fn parse_subnet(subnet: String) -> Result<Vec<Ipv4Addr>, Box<dyn Error>> {
    // Get the cidr of the subnet
    let cidr =
        u32::from_str(subnet.split('/').last().ok_or_else(|| {
            std::io::Error::new(ErrorKind::Other, "Failed to parse subnet mask")
        })?)?;

    // Get the subnet IP
    let ipaddr = subnet
        .split('/')
        .next()
        .ok_or_else(|| std::io::Error::new(ErrorKind::Other, "Failed to parse ip address"))?;

    // Convert the string IP into an integer
    let raw_ip: u32 = Ipv4Addr::from_str(ipaddr)?.into();

    // Get the IP range from the cidr
    let ip_range = 2u32.pow(32 - cidr);

    let mut ipaddrs: Vec<Ipv4Addr> = Vec::new();
    // Create a list of IP addresses given the IP range
    for mask in 0..ip_range {
        ipaddrs.push(Ipv4Addr::from(raw_ip | mask));
    }

    // Remove any duplicates
    ipaddrs.dedup();

    Ok(ipaddrs)
}
