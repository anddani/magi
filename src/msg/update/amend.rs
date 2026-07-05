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

use super::commit::{TOAST_DURATION, take_commit_author};

pub fn update(model: &mut Model, extra_args: Vec<String>) -> Option<Message> {
    // Dismiss the commit popup, keeping the author override it carries
    let author = take_commit_author(model);

    let mut flags: Vec<String> = vec![];

    flags.extend(extra_args);

    if let Some(CommitArguments(arguments)) = model.arguments.take() {
        flags.extend(arguments.into_iter().map(|a| a.flag().to_string()))
    };
    if let Some(author) = author {
        flags.push(format!("--author={}", author));
    }

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
