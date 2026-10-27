use crate::{
    git::file_checkout::get_modified_files,
    i18n,
    model::{
        Model,
        arguments::PopupArgument,
        popup::{MultiSelectPopupState, OnMultiSelect, PopupContent, PopupContentCommand},
    },
    msg::Message,
};

/// Opens the multi-select file picker for "Stash push".
///
/// The currently toggled stash-push argument flags (`-u`/`-a`/`-k`/`-K`) are
/// snapshotted into [`OnMultiSelect::StashPushFiles`] so they survive the
/// popup swap. If there are no modified files, an error popup is shown.
pub fn update(model: &mut Model) -> Option<Message> {
    let flags: Vec<String> = model
        .arguments
        .as_ref()
        .and_then(|a| a.stash_push())
        .map(|args| args.iter().map(|a| a.flag().to_string()).collect())
        .unwrap_or_default();

    let files = get_modified_files(&model.workdir);

    if files.is_empty() {
        model.popup = Some(PopupContent::error(
            i18n::t().error_no_modified_files.to_string(),
        ));
        return None;
    }

    let state = MultiSelectPopupState::new(
        i18n::t().title_stash_push_files.to_string(),
        files,
        OnMultiSelect::StashPushFiles { flags },
    );
    model.popup = Some(PopupContent::Command(PopupContentCommand::MultiSelect(
        state,
    )));
    None
}
