use std::time::Instant;

use crate::{
    git::commit::get_recent_commits_for_fixup,
    model::{
        Model, Toast, ToastStyle,
        popup::{PopupContent, PopupContentCommand, SelectContext, SelectPopupState},
    },
    msg::{FixupType, Message, update::commit::TOAST_DURATION},
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

    let repo_path = model.git_info.repository.workdir()?;

    match get_recent_commits_for_fixup(repo_path) {
        Ok(commits) => {
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
                let state = SelectPopupState::new(title, commits);
                model.popup = Some(PopupContent::Command(PopupContentCommand::Select(state)));
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
