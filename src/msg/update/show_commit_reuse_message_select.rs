use crate::{
    git::{checkout::get_branches, commit::rev_verify, push::get_local_tags},
    i18n,
    model::{
        Model,
        popup::{PopupContent, PopupContentCommand},
        select_popup::{OnSelect, SelectPopupState},
    },
    msg::Message,
};

/// Opens the reference picker for the commit `-C` argument
/// (`--reuse-message=<rev>`). The options are branches and tags, with
/// `ORIG_HEAD` first when it exists (matching magit's default). Arbitrary
/// revisions can still be typed since the picker falls back to the raw input
/// when nothing matches.
pub fn update(model: &mut Model) -> Option<Message> {
    let Some(PopupContent::Command(PopupContentCommand::Commit(mut state))) = model.popup.take()
    else {
        return None;
    };
    model.arg_mode = false;

    // Selecting the argument when a value is already set clears it
    if state.reuse_message.is_some() {
        state.reuse_message = None;
        model.popup = Some(PopupContent::Command(PopupContentCommand::Commit(state)));
        return None;
    }

    let mut options = get_branches(&model.git_info.repository);
    options.extend(get_local_tags(&model.git_info.repository));
    if rev_verify(&model.workdir, "ORIG_HEAD") {
        options.insert(0, "ORIG_HEAD".to_string());
    }

    let select_state = SelectPopupState::new(
        i18n::t().select_commit_reuse_message.to_string(),
        options,
        OnSelect::CommitReuseMessage {
            author: state.author,
        },
    );
    model.popup = Some(PopupContent::Command(PopupContentCommand::Select(
        select_state,
    )));
    None
}
