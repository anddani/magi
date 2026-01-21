use crate::{
    model::{
        popup::{PopupContent, PopupContentCommand, SelectResult},
        Model,
    },
    msg::Message,
};

pub fn update(model: &mut Model) -> Option<Message> {
    // If dismissing a select popup, mark it as cancelled
    if let Some(PopupContent::Command(PopupContentCommand::Select(_))) = &model.popup {
        model.select_result = Some(SelectResult::Cancelled);
    }
    model.popup = None;
    None
}
