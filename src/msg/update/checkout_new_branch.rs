use crate::{
    git::checkout::{CheckoutResult, checkout_new_branch, create_branch},
    model::{Model, popup::PopupContent},
    msg::Message,
};

pub fn update(
    model: &mut Model,
    starting_point: String,
    branch_name: String,
    checkout: bool,
) -> Option<Message> {
    let Some(repo_path) = model.git_info.repository.workdir() else {
        model.popup = Some(PopupContent::Error {
            message: "Cannot create branch: repository workdir not found".to_string(),
        });
        return None;
    };

    let result = if checkout {
        checkout_new_branch(repo_path, &branch_name, &starting_point)
    } else {
        create_branch(repo_path, &branch_name, &starting_point)
    };

    match result {
        Ok(CheckoutResult::Success) => {
            // Refresh to show the new branch state
            Some(Message::Refresh)
        }
        Ok(CheckoutResult::Error(err)) => {
            model.popup = Some(PopupContent::Error { message: err });
            None
        }
        Err(err) => {
            model.popup = Some(PopupContent::Error {
                message: format!("Failed to create branch: {}", err),
            });
            None
        }
    }
}
