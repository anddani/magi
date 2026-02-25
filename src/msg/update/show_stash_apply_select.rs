use crate::{
    model::{
        LineContent, Model,
        popup::{PopupContent, PopupContentCommand, SelectPopupState},
        select_popup::SelectContext,
    },
    msg::Message,
};

pub fn update(model: &mut Model) -> Option<Message> {
    // If cursor is on a stash line, apply it immediately
    let cursor_pos = model.ui_model.cursor_position;
    if let Some(line) = model.ui_model.lines.get(cursor_pos)
        && let LineContent::Stash(entry) = &line.content
    {
        return Some(Message::StashApply(format!("stash@{{{}}}", entry.index)));
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

    model.select_context = Some(SelectContext::ApplyStash);
    let state = SelectPopupState::new("Apply stash".to_string(), stashes);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Select(state)));
    None
}
