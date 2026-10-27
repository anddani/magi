use crate::{
    git::checkout::{SpinoffResult, spinoff_branch},
    i18n,
    model::{Model, popup::PopupContent},
    msg::Message,
};

pub fn update(model: &mut Model, branch_name: String) -> Option<Message> {
    match spinoff_branch(&model.workdir, &branch_name) {
        Ok(SpinoffResult::Success) => Some(Message::Refresh),
        Ok(SpinoffResult::Error(err)) => {
            model.popup = Some(PopupContent::Error { message: err });
            None
        }
        Err(err) => {
            model.popup = Some(PopupContent::Error {
                message: format!("{}: {err}", i18n::t().err_spinoff_failed),
            });
            None
        }
    }
}
