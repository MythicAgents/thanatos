use serde::Deserialize;
use thanatos_module::{send_data, Callback, MythicStatus};

#[derive(Deserialize)]
struct CdArgs {
    directory: String,
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

    match std::env::set_current_dir(params.directory) {
        Err(e) => send_data(
            MythicStatus::Error,
            Callback::Output,
            e.to_string().as_bytes(),
        )
        .unwrap(),
        _ => (),
    }
}
