use crate::{
    model::{
        Model,
        popup::{ConfirmAction, ConfirmPopupState, PopupContent},
    },
    msg::Message,
};

pub fn update(model: &mut Model, branch_name: String) -> Option<Message> {
    // Show confirmation popup
    model.popup = Some(PopupContent::Confirm(ConfirmPopupState {
        message: format!("Are you sure you want to delete '{}' (y/n)?", branch_name),
        on_confirm: ConfirmAction::DeleteBranch(branch_name),
    }));

    None
}
