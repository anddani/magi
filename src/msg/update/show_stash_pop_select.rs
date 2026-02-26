use crate::{
    model::{
        LineContent, Model,
        popup::{
            ConfirmAction, ConfirmPopupState, PopupContent, PopupContentCommand, SelectPopupState,
        },
        select_popup::SelectContext,
    },
    msg::Message,
};

pub fn update(model: &mut Model) -> Option<Message> {
    // If cursor is on a stash line, show confirmation for that specific stash
    let cursor_pos = model.ui_model.cursor_position;
    if let Some(line) = model.ui_model.lines.get(cursor_pos)
        && let LineContent::Stash(entry) = &line.content
    {
        let stash_ref = format!("stash@{{{}}}", entry.index);
        let message = format!(
            "Pop {}?",
            if entry.message.is_empty() {
                stash_ref.clone()
            } else {
                format!("{}: {}", stash_ref, entry.message)
            }
        );
        model.popup = Some(PopupContent::Confirm(ConfirmPopupState {
            message,
            on_confirm: ConfirmAction::PopStash(stash_ref),
        }));
        return None;
    }

    // Otherwise collect all stash entries and show a selection popup
    let stashes: Vec<String> = model
        .ui_model
        .lines
        .iter()
        .filter_map(|line| {
            if let LineContent::Stash(entry) = &line.content {
                Some(format!("stash@{{{}}}: {}", entry.index, entry.message))
            } else {
                None
            }
        })
        .collect();

    if stashes.is_empty() {
        model.popup = Some(PopupContent::Error {
            message: "No stashes found".to_string(),
        });
        return None;
    }

    model.select_context = Some(SelectContext::PopStash);
    let state = SelectPopupState::new("Pop stash".to_string(), stashes);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Select(state)));
    None
}
