use thanatos_module::{send_data, Callback, MythicStatus};

#[thanatos_module::entrypoint]
fn entrypoint(_parameters: Option<String>) {
    let cwd = std::env::current_dir().unwrap();
    let cwd = cwd.to_string_lossy();

    send_data(MythicStatus::Success, Callback::Output, cwd.as_bytes()).unwrap();
}
