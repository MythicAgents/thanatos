use std::sync::mpsc;

use agent_utils::msg::SocksMsg;

pub fn handle_socks(_tx: mpsc::Sender<SocksMsg>, rx: mpsc::Receiver<SocksMsg>) {
    let msg = match rx.recv() {
        Ok(msg) => msg,
        Err(_) => return,
    };

    agent_utils::log!("{:?}", msg);
}
