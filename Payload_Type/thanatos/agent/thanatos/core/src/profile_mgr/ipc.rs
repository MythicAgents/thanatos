use thanatos_protos::msg::AgentMessage;

pub enum ProfileIPCMsg {
    UpdateSleep { interval: u32, jitter: u32 },
    C2Data(AgentMessage),
}
