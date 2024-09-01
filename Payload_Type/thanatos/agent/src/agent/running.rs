use super::traits::{private::Sealed, AgentState};

pub struct Running;
impl Sealed for Running {}
impl AgentState for Running {}
