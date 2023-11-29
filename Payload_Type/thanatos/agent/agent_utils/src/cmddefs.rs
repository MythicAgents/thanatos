//! Definitions for commands. This is used for deserializing the Mythic JSON with the task information

use serde::Deserialize;

/// Maps the Mythic command to an enum value
#[derive(PartialEq, Deserialize, Debug)]
pub enum ThanatosCommand {
    /// Exit command
    #[serde(rename = "exit")]
    Exit,

    /// Sleep command
    #[serde(rename = "sleep")]
    Sleep,

    /// Link command
    #[serde(rename = "link")]
    Link,

    /// Pwd command
    #[serde(rename = "pwd")]
    Pwd,

    /// Cat command
    #[serde(rename = "cat")]
    Cat,

    /// Cd command
    #[serde(rename = "cd")]
    Cd,

    /// Set the working hours command
    #[serde(rename = "workinghours")]
    WorkingHours,

    /// Disable a C2 profile
    #[serde(rename = "disable-profile")]
    DisableProfile,

    /// Enable a C2 profile
    #[serde(rename = "enable-profile")]
    EnableProfile,

    /// Get the C2 profile settings
    #[serde(rename = "profiles")]
    Profiles,

    /// Load a command
    #[serde(rename = "load")]
    Load,

    /// Unload a command
    #[serde(rename = "unload")]
    Unload,

    /// Execution method
    #[serde(rename = "execution-method")]
    ExecutionMethod,

    /// Spawnto
    #[serde(rename = "spawnto")]
    SpawnTo,
}
