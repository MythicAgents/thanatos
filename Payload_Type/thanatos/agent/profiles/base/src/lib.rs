pub mod errors;

pub mod msg {
    pub mod checkin {
        include!(concat!(env!("OUT_DIR"), "/msg.checkin.rs"));
    }
}
