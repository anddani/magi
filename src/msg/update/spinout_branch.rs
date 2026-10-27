use crate::{
    git::checkout::{SpinoutResult, spinout_branch},
    i18n,
    model::{Model, popup::PopupContent},
    msg::Message,
};

pub fn update(model: &mut Model, branch_name: String) -> Option<Message> {
    match spinout_branch(&model.workdir, &branch_name) {
        Ok(SpinoutResult::Success) => Some(Message::Refresh),
        Ok(SpinoutResult::Error(err)) => {
            model.popup = Some(PopupContent::Error { message: err });
            None
        }
        Err(err) => {
            model.popup = Some(PopupContent::Error {
                message: format!("{}: {err}", i18n::t().err_spinout_failed),
            });
            None
        }
    }
}
