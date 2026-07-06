use crate::{
    model::{
        EditOp, Model,
        popup::{PopupContent, PopupContentCommand},
    },
    msg::Message,
};

pub fn update(model: &mut Model, op: EditOp) -> Option<Message> {
    if let Some(PopupContent::Command(PopupContentCommand::Select(state))) = &mut model.popup
        && state.input.apply(op)
    {
        state.update_filter();
    }
    None
}
