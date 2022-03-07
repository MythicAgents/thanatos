use crate::agent::AgentTask;
use crate::mythic_success;
use serde::Deserialize;
use std::error::Error;
use std::fs;

/// Struct containing the parameters for the cat command
#[derive(Deserialize)]
struct CatArgs {
    /// File to cat
    file: String,
}

/// Returns the contents of a specified file
/// * `task` - AgentTask structure containing the task information
pub fn cat_file(task: &AgentTask) -> Result<serde_json::Value, Box<dyn Error>> {
    // Parse the arguments
    let args: CatArgs = serde_json::from_str(&task.parameters)?;

    // Open the file and read it to a string
    let file_output = fs::read_to_string(&args.file)?;

    // Create the task response
    let mut output = mythic_success!(task.id, file_output);

    // Add a `FileOpen` artifact to the respone
    let output = output.as_object_mut().unwrap();
    output.insert(
        "artifacts".to_string(),
        serde_json::json!(
            [
                {
                    "base_artifact": "FileOpen",
                    "artifact": &args.file,
                }
            ]
        ),
    );

    // Return the output
    Ok(serde_json::to_value(output)?)
}
