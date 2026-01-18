use std::time::{Duration, Instant};

use crate::{
    git::push::{push_with_set_upstream, PushResult},
    model::{
        popup::{PopupContent, PopupContentCommand},
        Model, Toast, ToastStyle,
    },
    msg::Message,
};

/// Duration for toast notifications
const TOAST_DURATION: Duration = Duration::from_secs(5);

pub fn update(model: &mut Model) -> Option<Message> {
    // Get the branch name to push to
    let branch_name = if let Some(PopupContent::Command(PopupContentCommand::Push(ref state))) =
        model.popup
    {
        if state.input_text.is_empty() {
            // Use the local branch name as default
            state.local_branch.clone()
        } else {
            state.input_text.clone()
        }
    } else {
        return None;
    };

    // Dismiss the popup
    model.popup = None;

    if let Some(repo_path) = model.git_info.repository.workdir() {
        match push_with_set_upstream(repo_path, "origin", &branch_name) {
            Ok(PushResult { success, message }) => {
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
    }

    Some(Message::Refresh)
}
