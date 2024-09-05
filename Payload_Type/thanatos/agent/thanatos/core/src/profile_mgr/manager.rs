use std::thread::Scope;

use thanatos_protos::config::Config;

use crate::{errors::ThanatosError, log, profile_mgr::handler::ManagedProfile};

use super::{beacon::BeaconManager, handler::ProfileHandler};

pub struct ProfileManager {
    beacons: Option<BeaconManager>,
}

impl ProfileManager {
    pub fn new(config: &Config) -> ProfileManager {
        let beacons = BeaconManager::new(config);
        ProfileManager { beacons }
    }

    pub fn run<'scope, 'env: 'scope>(
        mut self,
        scope: &'scope Scope<'scope, 'env>,
    ) -> Result<ProfileHandler<'scope>, ThanatosError> {
        let (sender, receiver) = crossbeam_channel::unbounded();

        let beacons = self.beacons.take().map(|beacons| {
            let new_sender = sender.clone();
            let (profile_sender, profile_receiver) = crossbeam_channel::unbounded();
            ManagedProfile::new(
                profile_sender,
                scope.spawn(|| {
                    if let Err(e) = beacons.run(new_sender, profile_receiver) {
                        log!("{:?}", e);
                    }
                }),
            )
        });

        ProfileHandler::new(beacons, receiver)
    }
}
