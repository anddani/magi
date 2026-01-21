use crate::{
    git::checkout::{checkout, CheckoutResult},
    model::{popup::PopupContent, Model},
    msg::Message,
};

pub fn update(model: &mut Model, branch_name: String) -> Option<Message> {
    let Some(repo_path) = model.git_info.repository.workdir() else {
        model.popup = Some(PopupContent::Error {
            message: "Cannot checkout: repository workdir not found".to_string(),
        });
        return None;
    };

    match checkout(repo_path, &branch_name) {
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
                message: format!("Checkout failed: {}", err),
            });
            None
        }
    }
}
