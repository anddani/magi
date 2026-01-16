use std::time::{Duration, Instant};

use crate::{
    git::commit::{self, CommitResult},
    model::{popup::PopupContent, Model, Toast, ToastStyle},
    msg::Message,
};

/// Duration for toast notifications
pub const TOAST_DURATION: Duration = Duration::from_secs(5);

pub fn update(model: &mut Model) -> Option<Message> {
    if let Ok(false) = model.git_info.has_staged_changes() {
        model.toast = Some(Toast {
            message: "Nothing staged to commit".to_string(),
            style: ToastStyle::Warning,
            expires_at: Instant::now() + TOAST_DURATION,
        });
        return None;
    }

    if let Some(repo_path) = model.git_info.repository.workdir() {
        match commit::run_commit_with_editor(repo_path) {
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
    } else {
        model.popup = Some(PopupContent::Error {
            message: "Repository working directory not found".into(),
        });
    };
    Some(Message::Refresh)
}
