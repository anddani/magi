use crate::{
    model::{Model, ViewMode},
    msg::Message,
};

/// Exit log view and return to status view
pub fn update(model: &mut Model) -> Option<Message> {
    model.view_mode = ViewMode::Status;
    // Cancel any pending pick operation
    model.select_context = None;
    model.ui_model.visual_mode_anchor = None;
    // Trigger a refresh to reload the status view
    Some(Message::Refresh)
}
