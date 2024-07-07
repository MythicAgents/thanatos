pub mod config {
    include!(concat!(env!("OUT_DIR"), "/config.rs"));
}

pub mod msg {
    pub mod checkin {
        include!(concat!(env!("OUT_DIR"), "/msg.checkin.rs"));
    }
}
