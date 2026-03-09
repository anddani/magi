use std::time::Instant;

use crate::{
    git::push::{get_local_tags, get_remote_tags},
    model::{
        Model, Toast, ToastStyle,
        popup::{ConfirmAction, ConfirmPopupState, PopupContent},
    },
    msg::Message,
    msg::update::commit::TOAST_DURATION,
};

pub fn update(model: &mut Model, remote: String) -> Option<Message> {
    model.popup = None;

    let local_tags = get_local_tags(&model.git_info.repository);

    let remote_tags = match get_remote_tags(&model.workdir, &remote) {
        Some(tags) => tags,
        None => {
            model.popup = Some(PopupContent::Error {
                message: format!("Failed to fetch tags from remote '{}'", remote),
            });
            return None;
        }
    };

    // Tags that exist locally but not on the remote → delete locally
    let local_only: Vec<String> = local_tags
        .iter()
        .filter(|t| !remote_tags.contains(t))
        .cloned()
        .collect();

    // Tags that exist on the remote but not locally → push-delete from remote
    let remote_only: Vec<String> = remote_tags
        .iter()
        .filter(|t| !local_tags.contains(t))
        .cloned()
        .collect();

    if local_only.is_empty() && remote_only.is_empty() {
        model.toast = Some(Toast {
            message: format!("Same tags exist locally and on '{}'", remote),
            style: ToastStyle::Info,
            expires_at: Instant::now() + TOAST_DURATION,
        });
        return None;
    }

    let mut lines = vec![format!("Prune tags against '{}':", remote)];
    if !local_only.is_empty() {
        lines.push(format!("  Delete locally ({}):", local_only.len()));
        for tag in &local_only {
            lines.push(format!("    {}", tag));
        }
    }
    if !remote_only.is_empty() {
        lines.push(format!("  Delete from remote ({}):", remote_only.len()));
        for tag in &remote_only {
            lines.push(format!("    {}", tag));
        }
    }

    model.popup = Some(PopupContent::Confirm(ConfirmPopupState {
        message: lines.join("\n"),
        on_confirm: ConfirmAction::PruneTags {
            local_tags: local_only,
            remote_tags: remote_only,
            remote,
        },
    }));

    None
}
