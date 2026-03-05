use crate::model::{Model, ViewMode};
use crate::msg::Message;

pub fn update(model: &mut Model) -> Option<Message> {
    let return_mode = model.preview_return_mode.take().unwrap_or(ViewMode::Status);

    model.view_mode = return_mode;
    if let Some(ui_model) = model.preview_return_ui_model.take() {
        model.ui_model = ui_model;
    }
    Some(Message::Refresh)
}
