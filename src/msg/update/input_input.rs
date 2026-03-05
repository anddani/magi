use crate::{
    model::{
        Model,
        popup::{InputContext, PopupContent},
    },
    msg::{Message, StashCommand},
};

/// Handle a character input in the input popup
pub fn input_char(model: &mut Model, c: char) -> Option<Message> {
    if let Some(PopupContent::Input(ref mut state)) = model.popup {
        state.input_text.push(c);
    }
    None
}

/// Handle backspace in the input popup
pub fn input_backspace(model: &mut Model) -> Option<Message> {
    if let Some(PopupContent::Input(ref mut state)) = model.popup {
        state.input_text.pop();
    }
    None
}

/// Handle confirmation (Enter) in the input popup
pub fn confirm(model: &mut Model) -> Option<Message> {
    let Some(PopupContent::Input(state)) = model.popup.take() else {
        return None;
    };

    // Stash push allows empty input (git will use the default message)
    if let InputContext::Stash(stash_type) = state.context {
        return Some(Message::Stash(StashCommand::Push(
            stash_type,
            state.input_text.trim().to_string(),
        )));
    }

    let input = state.input_text.trim().to_string();
    if input.is_empty() {
        // Restore the popup if input is empty
        model.popup = Some(PopupContent::Input(state));
        return None;
    }

    match state.context {
        InputContext::CreateNewBranch {
            starting_point,
            checkout,
        } => Some(Message::CreateNewBranch {
            starting_point,
            branch_name: input,
            checkout,
        }),
        InputContext::RenameBranch { old_name } => Some(Message::RenameBranch {
            old_name,
            new_name: input,
        }),
        InputContext::SpinoffBranch => Some(Message::SpinoffBranch(input)),
        InputContext::WorktreePath { branch, checkout } => Some(Message::WorktreeCheckout {
            branch,
            path: input,
            checkout,
        }),
        InputContext::Stash(_) => unreachable!(),
    }
}
