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
    let cursor_pos = model.ui_model.cursor_position;
    let current_line = model.ui_model.lines.get(cursor_pos)?;

    // Check if cursor is on the Stashes section header
    if let LineContent::SectionHeader { title, .. } = &current_line.content
        && title == "Stashes"
    {
        // Show confirmation for dropping all stashes
        model.popup = Some(PopupContent::Confirm(ConfirmPopupState {
            message: "Drop all stashes?".to_string(),
            on_confirm: ConfirmAction::DropStash("all".to_string()),
        }));
        return None;
    }

    // If cursor is on a stash line, show confirmation for that specific stash
    if let LineContent::Stash(entry) = &current_line.content {
        let stash_ref = format!("stash@{{{}}}", entry.index);
        let message = format!(
            "Drop {}?",
            if entry.message.is_empty() {
                stash_ref.clone()
            } else {
                format!("{}: {}", stash_ref, entry.message)
            }
        );
        model.popup = Some(PopupContent::Confirm(ConfirmPopupState {
            message,
            on_confirm: ConfirmAction::DropStash(stash_ref),
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

    model.select_context = Some(SelectContext::DropStash);
    let state = SelectPopupState::new("Drop stash".to_string(), stashes);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Select(state)));
    None
}
