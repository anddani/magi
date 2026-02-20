use std::time::Instant;

use crate::{
    git::log::get_log_entries,
    model::{
        Model, Toast, ToastStyle,
        popup::{CommitSelectPopupState, PopupContent, PopupContentCommand, SelectContext},
    },
    msg::{FixupType, LogType, Message, update::commit::TOAST_DURATION},
};

pub fn update(model: &mut Model, fixup_type: FixupType) -> Option<Message> {
    // Check if there are staged changes
    if let Ok(false) = model.git_info.has_staged_changes() {
        model.toast = Some(Toast {
            message: "Nothing staged to fixup".to_string(),
            style: ToastStyle::Warning,
            expires_at: Instant::now() + TOAST_DURATION,
        });
        return Some(Message::DismissPopup);
    }

    match get_log_entries(&model.git_info.repository, LogType::Current) {
        Ok(mut commits) => {
            // Remove graph-only entries (keep only actual commits)
            commits.retain(|entry| entry.is_commit());

            // Limit to 50 commits for fixup selection
            commits.truncate(50);

            if commits.is_empty() {
                model.popup = Some(PopupContent::Error {
                    message: "No commits found in current branch".to_string(),
                });
                None
            } else {
                let title = match fixup_type {
                    FixupType::Fixup => "Fixup commit".to_string(),
                    FixupType::Squash => "Squash commit".to_string(),
                };
                let state = CommitSelectPopupState::new(title, commits);
                model.popup = Some(PopupContent::Command(PopupContentCommand::CommitSelect(
                    state,
                )));
                model.select_context = Some(SelectContext::FixupCommit(fixup_type));
                None
            }
        }
        Err(err) => {
            model.popup = Some(PopupContent::Error {
                message: format!("Failed to get recent commits: {}", err),
            });
            None
        }
    }
}
