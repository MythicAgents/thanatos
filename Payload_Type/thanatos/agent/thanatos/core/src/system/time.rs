use crate::log;

pub fn epoch_timestamp() -> u64 {
    match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
        Ok(ts) => ts.as_secs(),
        Err(_) => {
            log!("Failed to get system time");
            std::process::exit(0);
        }
    }
}
