use crate::{
    model::{Model, arguments::RebaseArgument},
    msg::{Message, OnSelect, OptionsSource, ShowSelectPopupConfig},
};

/// Toggles the `--rebase-merges=` argument: unsets it when set, otherwise
/// asks for the mode with a select popup (magit prompts between
/// no-rebase-cousins and rebase-cousins when enabling the argument).
pub fn update(model: &mut Model) -> Option<Message> {
    model.arg_mode = false;

    if let Some(args) = model.arguments.as_mut().and_then(|a| a.rebase_mut()) {
        let existing: Vec<RebaseArgument> = args
            .iter()
            .filter(|arg| matches!(arg, RebaseArgument::RebaseMerges(_)))
            .cloned()
            .collect();
        if !existing.is_empty() {
            for arg in existing {
                args.remove(&arg);
            }
            return None;
        }
    }

    Some(Message::ShowSelectPopup(ShowSelectPopupConfig {
        title: "Rebase merges".to_string(),
        source: OptionsSource::RebaseMergesModes,
        on_select: OnSelect::RebaseMergesMode,
    }))
}
