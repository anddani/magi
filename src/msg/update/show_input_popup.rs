use crate::{
    model::{
        Model,
        popup::{InputContext, PopupContent},
    },
    msg::Message,
};

pub fn update(model: &mut Model, context: InputContext) -> Option<Message> {
    model.popup = Some(PopupContent::input_popup(context));
    None
}
