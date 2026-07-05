use crate::{
    git::commit::list_authors,
    model::{
        Model,
        popup::{PopupContent, PopupContentCommand},
        select_popup::{OnSelect, SelectPopupState},
    },
    msg::Message,
};

pub fn update(model: &mut Model) -> Option<Message> {
    let Some(PopupContent::Command(PopupContentCommand::Commit(mut state))) = model.popup.take()
    else {
        return None;
    };
    model.arg_mode = false;

    // Selecting the argument when a value is already set clears it
    if state.author.is_some() {
        state.author = None;
        model.popup = Some(PopupContent::Command(PopupContentCommand::Commit(state)));
        return None;
    }

    let authors = list_authors(&model.workdir).unwrap_or_default();
    let select_state = SelectPopupState::new(
        "Commit author".to_string(),
        authors,
        OnSelect::CommitAuthor,
    );
    model.popup = Some(PopupContent::Command(PopupContentCommand::Select(
        select_state,
    )));
    None
}
