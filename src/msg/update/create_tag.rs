use std::process::Stdio;

use crate::{
    git::git_cmd,
    model::{
        Model,
        arguments::{Arguments::TagArguments, PopupArgument},
        popup::PopupContent,
    },
    msg::Message,
};

/// Create a new git tag pointing at `target`.
/// Equivalent to `git tag [--force] <name> <target>`.
pub fn update(model: &mut Model, name: String, target: String) -> Option<Message> {
    let extra_args: Vec<String> = if let Some(TagArguments(arguments)) = model.arguments.take() {
        arguments
            .into_iter()
            .map(|a| a.flag().to_string())
            .collect()
    } else {
        vec![]
    };

    let mut args = vec!["tag".to_string()];
    args.extend(extra_args);
    args.extend([name.clone(), target]);
    let args: Vec<&str> = args.iter().map(String::as_str).collect();

    let output = git_cmd(&model.workdir, &args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output();

    model.popup = None;

    match output {
        Ok(out) if out.status.success() => Some(Message::Refresh),
        Ok(out) => {
            let stderr = String::from_utf8_lossy(&out.stderr).trim().to_string();
            model.popup = Some(PopupContent::Error {
                message: format!("Failed to create tag '{}': {}", name, stderr),
            });
            None
        }
        Err(err) => {
            model.popup = Some(PopupContent::Error {
                message: format!("Failed to create tag '{}': {}", name, err),
            });
            None
        }
    }
}
