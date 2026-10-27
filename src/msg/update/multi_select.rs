use crate::{
    model::{
        EditOp, Model,
        popup::{OnMultiSelect, PopupContent, PopupContentCommand},
    },
    msg::{Message, StashCommand},
};

/// Returns a mutable reference to the active multi-select popup state, if any.
macro_rules! multi_select_state {
    ($model:expr) => {
        if let Some(PopupContent::Command(PopupContentCommand::MultiSelect(state))) =
            &mut $model.popup
        {
            Some(state)
        } else {
            None
        }
    };
}

pub fn edit(model: &mut Model, op: EditOp) -> Option<Message> {
    if let Some(state) = multi_select_state!(model)
        && state.input.apply(op)
    {
        state.update_filter();
    }
    None
}

pub fn move_up(model: &mut Model) -> Option<Message> {
    if let Some(state) = multi_select_state!(model) {
        state.move_up();
    }
    None
}

pub fn move_down(model: &mut Model) -> Option<Message> {
    if let Some(state) = multi_select_state!(model) {
        state.move_down();
    }
    None
}

pub fn toggle_current(model: &mut Model) -> Option<Message> {
    if let Some(state) = multi_select_state!(model) {
        state.toggle_current();
    }
    None
}

pub fn confirm(model: &mut Model) -> Option<Message> {
    let Some(PopupContent::Command(PopupContentCommand::MultiSelect(state))) = model.popup.take()
    else {
        return None;
    };

    let files = state.checked_items();
    match state.on_confirm {
        OnMultiSelect::StashPushFiles { flags } => {
            Some(Message::Stash(StashCommand::PushFiles { files, flags }))
        }
    }
}
