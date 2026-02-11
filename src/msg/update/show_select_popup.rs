use crate::{
    model::{
        Model,
        popup::{PopupContent, PopupContentCommand, SelectPopupState},
    },
    msg::Message,
};

pub fn update(model: &mut Model, title: String, options: Vec<String>) -> Option<Message> {
    let state = SelectPopupState::new(title, options);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Select(state)));
    None
}
