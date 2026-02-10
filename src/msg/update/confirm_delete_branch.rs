use crate::{
    git::checkout::{delete_branch, DeleteBranchResult},
    model::{popup::PopupContent, Model},
    msg::Message,
};

pub fn update(model: &mut Model, branch_name: String) -> Option<Message> {
    let Some(repo_path) = model.git_info.repository.workdir() else {
        model.popup = Some(PopupContent::Error {
            message: "Cannot delete branch: repository workdir not found".to_string(),
        });
        return None;
    };

    match delete_branch(repo_path, &branch_name) {
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
