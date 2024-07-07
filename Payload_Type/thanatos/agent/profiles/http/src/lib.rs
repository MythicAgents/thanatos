pub struct HttpC2Profile {
    uuid: String,
    _callback_host: String,
    _callback_port: u32,
}

impl HttpC2Profile {
    pub fn new(_uuid: utils::uuid::Uuid) -> HttpC2Profile {
        todo!();
    }

    pub fn uuid(&self) -> &str {
        &self.uuid
    }

    pub fn send_checkin(&mut self) {
        todo!();
    }
}
