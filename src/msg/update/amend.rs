use std::time::Instant;

use crate::{
    git::commit::{self, CommitResult},
    model::{Model, Toast, ToastStyle, popup::PopupContent},
    msg::Message,
};

use super::commit::TOAST_DURATION;

pub fn update(model: &mut Model) -> Option<Message> {
    // Dismiss any open popup (e.g., commit popup)
    model.popup = None;

    match commit::run_amend_commit_with_editor(&model.workdir) {
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
