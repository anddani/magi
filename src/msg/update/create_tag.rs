use std::process::Stdio;

use crate::{
    git::git_cmd,
    model::{
        Model,
        arguments::{Arguments::TagArguments, PopupArgument, TagArgument},
        popup::PopupContent,
    },
    msg::Message,
};

/// Consumes the tag popup arguments, returning their git flags in a
/// stable order (--force before --edit before --annotate before --sign).
fn take_tag_flags(model: &mut Model) -> Vec<String> {
    if let Some(TagArguments(arguments)) = model.arguments.take() {
        TagArgument::all()
            .into_iter()
            .filter(|arg| arguments.contains(arg))
            .map(|arg| arg.flag().to_string())
            .collect()
    } else {
        vec![]
    }
}

/// Create a new git tag pointing at `target`.
/// Equivalent to `git tag [--force] [--edit] [--annotate] [--sign] <name> <target>`.
///
/// With --edit, --annotate or --sign, `git tag` opens the editor for the
/// tag message, so the command must run with the TUI suspended instead of
/// capturing output.
pub fn update(model: &mut Model, name: String, target: String) -> Option<Message> {
    let flags = take_tag_flags(model);

    let mut args = vec!["tag".to_string()];
    args.extend(flags.iter().cloned());
    args.extend([name.clone(), target]);

    if flags
        .iter()
        .any(|flag| flag == "--edit" || flag == "--annotate" || flag == "--sign")
    {
        return Some(Message::CreateTagWithEditor { name, args });
    }

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

/// Runs `git tag --edit/--annotate/--sign ...` with stdio inherited so the user's
/// configured editor can open for the tag message. Requires the TUI to be
/// suspended.
pub fn with_editor(model: &mut Model, name: String, args: Vec<String>) -> Option<Message> {
    model.popup = None;

    let args: Vec<&str> = args.iter().map(String::as_str).collect();
    let status = git_cmd(&model.workdir, &args).status();

    match status {
        Ok(status) if status.success() => Some(Message::Refresh),
        Ok(_) => {
            model.popup = Some(PopupContent::Error {
                message: format!("Failed to create tag '{}'", name),
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
