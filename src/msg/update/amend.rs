use std::time::Instant;

use crate::{
    git::commit::{self, CommitResult},
    model::{
        Model, Toast, ToastStyle,
        arguments::{Arguments::CommitArguments, PopupArgument},
        popup::PopupContent,
    },
    msg::Message,
};

use super::commit::{TOAST_DURATION, check_signing, push_value_flags, take_commit_popup_state};

pub fn update(model: &mut Model, extra_args: Vec<String>) -> Option<Message> {
    // Dismiss the commit popup, keeping the value arguments it carries
    let popup_state = take_commit_popup_state(model);

    if let Some(message) = check_signing(model) {
        model.popup = Some(PopupContent::Error { message });
        return None;
    }

    let mut flags: Vec<String> = vec![];

    flags.extend(extra_args);

    if let Some(CommitArguments(arguments)) = model.arguments.take() {
        flags.extend(arguments.into_iter().map(|a| a.flag().to_string()))
    };
    push_value_flags(&mut flags, popup_state);

    match commit::run_amend_commit_with_editor(&model.workdir, flags) {
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
