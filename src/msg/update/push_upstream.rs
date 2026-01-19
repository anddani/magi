use std::time::{Duration, Instant};

use crate::{
    git::push::{push_to_upstream, PushResult},
    model::{popup::PopupContent, Model, Toast, ToastStyle},
    msg::Message,
};

/// Duration for toast notifications
const TOAST_DURATION: Duration = Duration::from_secs(5);

pub fn update(model: &mut Model) -> Option<Message> {
    // Dismiss the popup
    model.popup = None;

    if let Some(repo_path) = model.git_info.repository.workdir() {
        match push_to_upstream(repo_path) {
            Ok(PushResult { success, message }) => {
                if success {
                    model.toast = Some(Toast {
                        message,
                        style: ToastStyle::Success,
                        expires_at: Instant::now() + TOAST_DURATION,
                    });
                } else {
                    // Show error popup with git output
                    model.popup = Some(PopupContent::Error { message });
                }
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
