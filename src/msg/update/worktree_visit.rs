use std::path::PathBuf;

use crate::{
    i18n,
    model::{Model, popup::PopupContent},
    msg::Message,
    msg::update::worktree_checkout::switch_to_worktree,
};

pub fn update(model: &mut Model, worktree: String) -> Option<Message> {
    let path = PathBuf::from(&worktree);
    if !path.exists() {
        model.popup = Some(PopupContent::Error {
            message: i18n::t().fmt1(i18n::t().err_worktree_does_not_exist_fmt, &worktree),
        });
        return None;
    }
    switch_to_worktree(model, path);
    Some(Message::Refresh)
}
