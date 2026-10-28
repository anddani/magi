use crate::{
    git::commit::list_authors,
    i18n,
    model::{
        Model,
        popup::{PopupContent, PopupContentCommand},
        select_popup::{OnSelect, SelectPopupState},
    },
    msg::Message,
};

/// Open the author picker for the `-A` log argument. The authors are gathered
/// from the commit history and offered as a fuzzy-filterable list (a typed
/// pattern that matches nothing is still used as the `--author=` value). The
/// author lives in `model.arguments`, so selecting one writes it back and
/// reopens the log popup. Pressing `-A` again while an author is set clears it.
pub fn update(model: &mut Model) -> Option<Message> {
    model.arg_mode = false;

    // Selecting the argument when a value is already set clears it
    if let Some(author) = model.arguments.as_mut().and_then(|a| a.log_author_mut())
        && author.is_some()
    {
        *author = None;
        return None;
    }

    let authors = list_authors(&model.workdir).unwrap_or_default();
    let select_state = SelectPopupState::new(
        i18n::t().select_log_author.to_string(),
        authors,
        OnSelect::LogAuthor,
    );
    model.popup = Some(PopupContent::Command(PopupContentCommand::Select(
        select_state,
    )));
    None
}
