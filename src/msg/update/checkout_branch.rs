use crate::{
    git::checkout::{CheckoutResult, checkout},
    i18n,
    model::{Model, popup::PopupContent},
    msg::Message,
};

pub fn update(model: &mut Model, branch_name: String) -> Option<Message> {
    match checkout(&model.workdir, &branch_name) {
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
                message: format!("{}: {}", i18n::t().err_checkout_failed, err),
            });
            None
        }
    }
}
