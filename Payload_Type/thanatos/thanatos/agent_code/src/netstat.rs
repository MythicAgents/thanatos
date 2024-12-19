use serde::Serialize;
use crate::agent::AgentTask;
use crate::mythic_success;
use netstat2::{get_sockets_info, AddressFamilyFlags, ProtocolFlags, ProtocolSocketInfo};

/// Struct holding the information for network connections
#[derive(Default, Serialize)]
pub struct NetworkListingEntry {
    /// Protocol
    pub proto: String,

    /// Local address
    pub local_addr: String,

    /// Local Port
    pub local_port: u16,

    /// Remote address
    pub remote_addr: Option<String>,

    /// Remote port
    pub remote_port: Option<u16>,

    /// Associated PIDs
    pub associated_pids: Vec<u32>,

    /// State
    pub state: Option<String>,
}

pub fn netstat(task: &AgentTask) -> Result<(serde_json::Value), Box<dyn std::error::Error>> { 
    let af_flags = AddressFamilyFlags::IPV4 | AddressFamilyFlags::IPV6;
    let proto_flags = ProtocolFlags::TCP | ProtocolFlags::UDP;
    let sockets_info = get_sockets_info(af_flags, proto_flags)?;

    let mut conn: Vec<NetworkListingEntry> = Vec::new();

    for si in sockets_info {
        match si.protocol_socket_info {
            ProtocolSocketInfo::Tcp(tcp_si) => conn.push(NetworkListingEntry {
                proto: "TCP".to_string(),
                local_addr: tcp_si.local_addr.to_string(),
                local_port: tcp_si.local_port,
                remote_addr: Some(tcp_si.remote_addr.to_string()),
                remote_port: Some(tcp_si.remote_port),
                associated_pids: si.associated_pids,
                state: Some(tcp_si.state.to_string()),
            }),
            ProtocolSocketInfo::Udp(udp_si) => conn.push(NetworkListingEntry {
                proto: "UDP".to_string(),
                local_addr: udp_si.local_addr.to_string(),
                local_port: udp_si.local_port,
                remote_addr: None,
                remote_port: None,
                associated_pids: si.associated_pids,
                state: None,
            }),
        }
    }

    let user_output = serde_json::to_string(&conn)?;
    /// Return the output to Mythic
    Ok(mythic_success!(task.id, user_output))
}
