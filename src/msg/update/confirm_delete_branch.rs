use crate::{
    git::checkout::{DeleteBranchResult, delete_branch},
    model::{Model, popup::PopupContent},
    msg::Message,
};

pub fn update(model: &mut Model, branch_name: String) -> Option<Message> {
    match delete_branch(&model.workdir, &branch_name) {
        Ok(DeleteBranchResult::Success) => {
            model.popup = None;
            Some(Message::Refresh)
        }
        Ok(DeleteBranchResult::Error(err)) => {
            model.popup = Some(PopupContent::Error { message: err });
            None
        }
        Err(err) => {
            model.popup = Some(PopupContent::Error {
                message: format!("Delete branch failed: {}", err),
            });
            None
        }
    }
}
