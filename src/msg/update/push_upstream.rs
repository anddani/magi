use std::time::{Duration, Instant};

use crate::{
    git::push::{push, PushResult},
    model::{popup::PopupContent, Model, Toast, ToastStyle},
    msg::Message,
};

/// Duration for toast notifications
const TOAST_DURATION: Duration = Duration::from_secs(5);

pub fn update(model: &mut Model) -> Option<Message> {
    // Dismiss the popup
    model.popup = None;

    if let Some(repo_path) = model.git_info.repository.workdir() {
        match push(repo_path, &[]) {
            Ok(PushResult::Success) => {
                model.toast = Some(Toast {
                    message: "Pushed to upstream".to_string(),
                    style: ToastStyle::Success,
                    expires_at: Instant::now() + TOAST_DURATION,
                });
            }
            Ok(PushResult::Error(message)) => {
                model.popup = Some(PopupContent::Error { message });
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
