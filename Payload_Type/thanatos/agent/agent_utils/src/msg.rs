//! Definitions for Mythic messages

use crate::cmddefs::ThanatosCommand;

use super::CheckinInfo;
use serde::{Deserialize, Serialize};

/// Action types for Mythic messages.
#[derive(Serialize, Deserialize, Debug, Default)]
pub enum MythicAction {
    /// Mythic "checkin" action.
    #[default]
    #[serde(rename = "checkin")]
    Checkin,

    /// Mythic "get_tasking" action.
    #[serde(rename = "get_tasking")]
    GetTasking,

    /// Posts responses
    #[serde(rename = "post_response")]
    PostResponse,

    /// Stage an EKE
    #[serde(rename = "staging_rsa")]
    StagingRsa,
}

/// Status value from Mythic
#[derive(Serialize, Deserialize, PartialEq, Default, Debug)]
pub enum MythicStatus {
    /// Mythic success
    #[serde(rename = "success")]
    #[default]
    Success,

    /// Mythic had an error
    #[serde(rename = "error")]
    Error,

    /// Mythic completed the task
    #[serde(rename = "completed")]
    Completed,
}

impl MythicStatus {
    /// Converts a u8 into a `MythicStatus`
    pub fn from_u8(val: u8) -> Self {
        match val {
            0 => MythicStatus::Success,
            1 => MythicStatus::Error,
            _ => MythicStatus::Completed,
        }
    }
}

/// Mythic message for the initial checkin
#[derive(Serialize, Debug)]
pub struct InitialCheckinInfo<'s> {
    /// Action to be performed ("checkin")
    pub action: MythicAction,

    /// UUID of the agent
    pub uuid: &'s str,

    /// OS information for the checkin
    #[serde(flatten)]
    pub attributes: CheckinInfo,

    /// Extra info to send in the checkin
    pub extra_info: &'s str,

    /// Agent's sleep information
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sleep_info: Option<&'s str>,
}

/// C2 Profile information for Mythic
#[derive(Serialize, Debug)]
pub struct ExtraInfoC2Profile<'name> {
    /// ID for the C2 profile
    pub id: usize,

    /// C2 profile name
    pub name: &'name str,

    /// Whether the profile is enabled
    pub enabled: bool,

    /// Whether the profile is defunct
    pub defunct: bool,
}

/// Agent's configured spawnto value
#[derive(Serialize, Debug, Clone)]
pub struct SpawnToValue {
    /// The spawnto path
    pub path: String,
    /// The spawnto command line arguments
    pub args: Vec<String>,
}

/// Extra info to send in the callback
#[derive(Serialize, Debug)]
pub struct ExtraInfo<'a> {
    /// Formatted working hours
    pub working_hours: &'a str,

    /// Whether loaded commands should execute internally
    pub exec_internal: bool,

    /// Configured C2 profiles
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub c2_profiles: Vec<ExtraInfoC2Profile<'a>>,

    /// Initial spawnto value
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spawnto: Option<SpawnToValue>,
}

/// Message for performing an encrypted key exchange
#[cfg(feature = "EKE")]
#[derive(Serialize, Debug)]
pub struct StagingEKEMessage<'a> {
    /// Action to perform ("staging_rsa")
    pub action: MythicAction,

    /// RSA public key for the EKE
    pub pub_key: String,

    /// Random 20 character session key
    pub session_id: &'a str,
}

/// Response for an encrypted key exchange request
#[cfg(feature = "EKE")]
#[derive(Deserialize, Debug, Default)]
pub struct StagingEKEResponse {
    /// Action being perfomed ("staging_rsa")
    pub action: MythicAction,

    /// New UUID for the next message
    pub uuid: String,

    /// Base64 encoded crypto key for the next message
    pub session_key: String,

    /// 20 character session key
    pub session_id: String,
}

/// Response from Mythic after the initial checkin
#[derive(Deserialize, Default, Debug)]
pub struct InitialCheckinResponse {
    /// Action being performed
    pub action: MythicAction,

    /// New UUID to use
    pub id: String,

    /// Status of the checkin
    pub status: MythicStatus,
}

/// Message for getting new tasking from Mythic
#[derive(Serialize, Debug)]
pub struct GetTaskingMsg {
    /// Action being performed
    pub action: MythicAction,

    /// Number of tasks to get from Mythic
    pub tasking_size: isize,

    /// Completed task responses
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub responses: Vec<CompletedTask>,

    /// Delegate messages
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub delegates: Vec<DelegateMessage>,

    /// Mythic socks messages
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub socks: Vec<SocksMsg>,
}

impl GetTaskingMsg {
    /// Constructs a new `GetTaskingMsg` object
    pub fn new() -> Self {
        Self {
            action: MythicAction::GetTasking,
            tasking_size: -1,
            responses: Vec::new(),
            delegates: Vec::new(),
            socks: Vec::new(),
        }
    }

    /// Construct a new `GetTaskingMsg` object with completed tasking
    pub fn with_responses(
        responses: Vec<CompletedTask>,
        delegates: Vec<DelegateMessage>,
        socks: Vec<SocksMsg>,
    ) -> Self {
        Self {
            responses,
            delegates,
            socks,
            action: MythicAction::GetTasking,
            tasking_size: -1,
        }
    }
}

impl Default for GetTaskingMsg {
    fn default() -> Self {
        Self::new()
    }
}

/// Response for `get_tasking` requests
#[derive(Deserialize, Debug, Default)]
pub struct GetTaskingResponse {
    /// Action being performed ("get_tasking")
    pub action: MythicAction,

    /// Pending tasks
    #[serde(default)]
    pub tasks: Vec<PendingTask>,

    /// Delegates
    #[serde(default)]
    pub delegates: Vec<DelegateMessage>,

    /// Socks
    #[serde(default)]
    pub socks: Vec<SocksMsg>,
}

/// Message containing socks info
#[derive(Serialize, Deserialize, Debug)]
pub struct SocksMsg {
    /// Whether the sock server id should exit
    pub exit: bool,

    /// Socks server id
    pub server_id: usize,

    /// Data for the socks message
    pub data: String,
}

/// Tasks received from Mythic which need to be processed
#[derive(Deserialize, Debug)]
pub struct PendingTask {
    /// Task command
    pub command: ThanatosCommand,

    /// Command parameters
    pub parameters: String,

    /// Timestamp
    pub timestamp: f64,

    /// UUID of the task
    pub id: String,
}

/// C2 profile name for delegate message
#[derive(Serialize, Deserialize, Debug, Default)]
pub enum DelegateC2ProfileName {
    /// TCP p2p delegate
    #[serde(rename = "tcp")]
    #[default]
    Tcp,

    /// SMB p2p delegate
    #[serde(rename = "smb")]
    Smb,
}

/// P2P message
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct DelegateMessage {
    /// P2P message
    pub message: String,

    /// UUID of the agent for the p2p message
    pub uuid: String,

    /// C2 profile for p2p message
    #[serde(skip_serializing_if = "Option::is_none")]
    pub c2_profile: Option<DelegateC2ProfileName>,

    /// UUID mythic uses
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mythic_uuid: Option<String>,
}

/// Completed task information
#[derive(Serialize, Default, Debug)]
pub struct CompletedTask {
    /// UUID of the task
    pub task_id: String,

    /// Task result information
    #[serde(flatten)]
    pub task_attributes: TaskResults,
}

/// Completed tasking result info
#[derive(Serialize, Default, Debug)]
pub struct TaskResults {
    /// Status of the task
    pub status: MythicStatus,

    /// Whether the command has fully completed
    pub completed: bool,

    /// Any user output to reflect to Mythic
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_output: Option<String>,

    /// Any error messages
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,

    /// Anything to pass to `process_response`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub process_response: Option<String>,

    /// Link information
    #[serde(skip_serializing_if = "Option::is_none")]
    pub edges: Option<Vec<LinkEdge>>,
}

/// New link information
#[derive(Serialize, Default, Debug)]
pub struct LinkEdge {
    /// Uuid of the source callback
    source: String,

    /// Uuid of the destination callback
    destination: String,

    /// Optional metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    metadata: Option<String>,

    /// Action for the link
    action: LinkAction,

    /// Name of the C2 profile
    c2_profile: String,
}

/// Action to be performed when linking (add, remove)
#[derive(Serialize, Default, Debug)]
#[allow(unused)]
enum LinkAction {
    /// New link
    #[default]
    #[serde(rename = "add")]
    Add,

    /// Remove and existing link
    #[serde(rename = "remove")]
    Remove,
}
