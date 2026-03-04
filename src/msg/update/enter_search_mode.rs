use crate::{model::Model, msg::Message};

pub fn update(model: &mut Model) -> Option<Message> {
    model.ui_model.search_query.clear();
    model.ui_model.search_mode_active = true;
    None
}
