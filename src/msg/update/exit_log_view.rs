use crate::{
    model::{Model, ViewMode},
    msg::Message,
};

/// Exit log view and return to status view
pub fn update(model: &mut Model) -> Option<Message> {
    model.view_mode = ViewMode::Status;
    // Trigger a refresh to reload the status view
    Some(Message::Refresh)
}
