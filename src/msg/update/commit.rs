use std::time::{Duration, Instant};

use crate::{
    git::commit::{self, CommitResult},
    model::{
        Model, Toast, ToastStyle,
        arguments::{Arguments::CommitArguments, CommitArgument, PopupArgument},
        popup::{PopupContent, PopupContentCommand},
    },
    msg::Message,
};

/// Duration for toast notifications
pub const TOAST_DURATION: Duration = Duration::from_secs(5);

/// Dismisses the commit popup and returns the author override (`-A`) it held,
/// if any.
pub fn take_commit_author(model: &mut Model) -> Option<String> {
    match model.popup.take() {
        Some(PopupContent::Command(PopupContentCommand::Commit(state))) => state.author,
        _ => None,
    }
}

pub fn update(model: &mut Model) -> Option<Message> {
    // Dismiss the commit popup, keeping the author override it carries
    let author = take_commit_author(model);

    let allow_no_staged: bool = if let Some(CommitArguments(ref args)) = model.arguments {
        args.contains(&CommitArgument::StageAll) || args.contains(&CommitArgument::AllowEmpty)
    } else {
        false
    };

    // If argument allowing no staged files is selected, we want to allow the user to not have anything staged
    if !allow_no_staged && let Ok(false) = model.git_info.has_staged_changes() {
        model.toast = Some(Toast {
            message: "Nothing staged to commit".to_string(),
            style: ToastStyle::Warning,
            expires_at: Instant::now() + TOAST_DURATION,
        });
        return None;
    }

    let repo_path = &model.workdir;

    let mut flags: Vec<String> = if let Some(CommitArguments(arguments)) = model.arguments.take() {
        arguments
            .into_iter()
            .map(|a| a.flag().to_string())
            .collect()
    } else {
        vec![]
    };
    if let Some(author) = author {
        flags.push(format!("--author={}", author));
    }

    match commit::run_commit_with_editor(repo_path, flags) {
        Ok(CommitResult { success, message }) => {
            model.toast = Some(Toast {
                message,
                style: if success {
                    ToastStyle::Success
                } else {
                    ToastStyle::Warning
                },
                expires_at: Instant::now() + TOAST_DURATION,
            });
        }
        Err(e) => {
            model.popup = Some(PopupContent::Error {
                message: e.to_string(),
            });
        }
    }

    Some(Message::Refresh)
}
