use std::time::{Duration, Instant};

use crate::{
    git::commit::{self, CommitResult},
    model::{
        Model, Toast, ToastStyle,
        arguments::{Arguments::CommitArguments, CommitArgument, PopupArgument},
        popup::{CommitPopupState, PopupContent, PopupContentCommand},
    },
    msg::Message,
};

/// Duration for toast notifications
pub const TOAST_DURATION: Duration = Duration::from_secs(5);

/// Dismisses the commit popup and returns the value arguments (`-A` author
/// and `-C` reuse-message) it held.
pub fn take_commit_popup_state(model: &mut Model) -> CommitPopupState {
    match model.popup.take() {
        Some(PopupContent::Command(PopupContentCommand::Commit(state))) => state,
        _ => CommitPopupState::default(),
    }
}

/// Appends the flags for the commit popup's value arguments (`--author=` and
/// `--reuse-message=`) to the given flag list.
pub fn push_value_flags(flags: &mut Vec<String>, state: CommitPopupState) {
    if let Some(author) = state.author {
        flags.push(format!("--author={}", author));
    }
    if let Some(rev) = state.reuse_message {
        flags.push(format!("--reuse-message={}", rev));
    }
}

/// When the gpg-sign argument is selected, verifies that git can actually
/// sign before the editor opens, so a missing or broken key surfaces
/// immediately instead of aborting the commit after the message is written.
/// Returns the error message to show when signing does not work.
pub fn check_signing(model: &Model) -> Option<String> {
    let signing = matches!(
        &model.arguments,
        Some(CommitArguments(args)) if args.contains(&CommitArgument::GpgSign)
    );
    if !signing {
        return None;
    }
    match commit::signing_error(&model.workdir) {
        Ok(None) => None,
        Ok(Some(err)) => Some(format!("Cannot sign commit:\n\n{}", err)),
        Err(e) => Some(format!("Cannot sign commit:\n\n{}", e)),
    }
}

pub fn update(model: &mut Model) -> Option<Message> {
    // Dismiss the commit popup, keeping the value arguments it carries
    let popup_state = take_commit_popup_state(model);

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

    if let Some(message) = check_signing(model) {
        model.popup = Some(PopupContent::Error { message });
        return None;
    }

    let mut flags: Vec<String> = if let Some(CommitArguments(arguments)) = model.arguments.take() {
        arguments
            .into_iter()
            .map(|a| a.flag().to_string())
            .collect()
    } else {
        vec![]
    };
    push_value_flags(&mut flags, popup_state);

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_push_value_flags_empty_state_adds_nothing() {
        let mut flags = vec!["--verbose".to_string()];
        push_value_flags(&mut flags, CommitPopupState::default());
        assert_eq!(flags, vec!["--verbose".to_string()]);
    }

    #[test]
    fn test_push_value_flags_adds_author_and_reuse_message() {
        let mut flags = vec![];
        push_value_flags(
            &mut flags,
            CommitPopupState {
                author: Some("Jane Doe <jane@example.com>".to_string()),
                reuse_message: Some("ORIG_HEAD".to_string()),
            },
        );
        assert_eq!(
            flags,
            vec![
                "--author=Jane Doe <jane@example.com>".to_string(),
                "--reuse-message=ORIG_HEAD".to_string(),
            ]
        );
    }
}
