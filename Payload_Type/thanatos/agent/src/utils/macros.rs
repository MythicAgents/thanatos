/// Macro used for formatting a string into the Mythic error json format
/// * `task_id` - Id of the task
/// * `error` - Error to display in the Mythic task
#[macro_export]
macro_rules! mythic_error {
    ($tid:expr, $errmsg:expr) => {
        serde_json::json!({
            "task_id": $tid,
            "status": "error",
            "user_output": $errmsg,
            "completed": true,
        })
    };
}

/// Macro used for formatting a string into the Mythic success json format
/// * `task_id` - Id of the task
/// * `output` - Message for the task output
#[macro_export]
macro_rules! mythic_success {
    ($tid:expr, $out:expr) => {
        serde_json::json!({
            "task_id": $tid,
            "status": "success",
            "user_output": $out,
            "completed": true,
        })
    };
}

/// Macro used for formatting a string into the Mythic json format with a custom status message
/// * `task_id` - Id of the task
/// * `status` - Status message for the task
/// * `output` - Message for the task output
#[macro_export]
macro_rules! mythic_continued {
    ($tid:expr, $stat:expr, $out:expr) => {
        serde_json::json!({
            "task_id": $tid,
            "status": $stat,
            "user_output": $out,
            "completed": false,
        })
    };
}
