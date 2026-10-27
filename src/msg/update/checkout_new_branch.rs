use crate::{
    git::checkout::{CheckoutResult, checkout_new_branch, create_branch},
    i18n,
    model::{Model, popup::PopupContent},
    msg::Message,
};

pub fn update(
    model: &mut Model,
    starting_point: String,
    branch_name: String,
    checkout: bool,
) -> Option<Message> {
    let result = if checkout {
        checkout_new_branch(&model.workdir, &branch_name, &starting_point)
    } else {
        create_branch(&model.workdir, &branch_name, &starting_point)
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
                message: format!("{}: {}", i18n::t().err_create_branch_failed, err),
            });
            None
        }
    }
}
