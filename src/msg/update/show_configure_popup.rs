use crate::{
    git::push::get_current_branch,
    i18n,
    model::{
        Model,
        popup::{ConfigurePopupState, PopupContent, PopupContentCommand},
        select_popup::{OnSelect, OptionsSource},
    },
    msg::{Message, ShowSelectPopupConfig},
};

/// Open the Configure popup (git-config editor). When `force_pick` is true (the
/// branch popup's `C`) or HEAD is detached, a local-branch picker is shown
/// first; otherwise the popup is scoped to the current branch. Mirrors magit's
/// `magit-branch-configure` scope selection.
pub fn update(model: &mut Model, force_pick: bool) -> Option<Message> {
    let current = get_current_branch(&model.workdir).ok().flatten();

    if force_pick || current.is_none() {
        return Some(Message::ShowSelectPopup(ShowSelectPopupConfig {
            title: i18n::t().title_configure_branch.to_string(),
            source: OptionsSource::LocalBranches,
            on_select: OnSelect::ConfigureBranch,
        }));
    }

    let branch = current.unwrap_or_default();
    model.arg_mode = false;
    model.popup = Some(PopupContent::Command(PopupContentCommand::Configure(
        ConfigurePopupState {
            branch,
            pending_a: false,
        },
    )));
    None
}
