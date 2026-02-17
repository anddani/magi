use crate::{
    git::checkout::{RenameBranchResult, rename_branch},
    model::{Model, popup::PopupContent},
    msg::Message,
};

pub fn update(model: &mut Model, old_name: String, new_name: String) -> Option<Message> {
    match rename_branch(&model.workdir, &old_name, &new_name) {
        Ok(RenameBranchResult::Success) => {
            model.popup = None;
            Some(Message::Refresh)
        }
        Ok(RenameBranchResult::Error(err)) => {
            model.popup = Some(PopupContent::Error { message: err });
            None
        }
        Err(err) => {
            model.popup = Some(PopupContent::Error {
                message: format!("Rename branch failed: {}", err),
            });
            None
        }
    }
}
