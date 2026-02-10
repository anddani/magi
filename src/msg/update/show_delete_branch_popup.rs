use crate::{
    git::checkout::get_branches,
    model::{
        popup::{PopupContent, PopupContentCommand, SelectPopupState},
        select_popup::SelectContext,
        suggestions_from_line, BranchSuggestion, Model,
    },
    msg::Message,
};

pub fn update(model: &mut Model) -> Option<Message> {
    let mut branches: Vec<String> = get_branches(&model.git_info.repository);

    if branches.is_empty() {
        model.popup = Some(PopupContent::Error {
            message: "No branches found".to_string(),
        });
        return None;
    }

    // Set the context so select_confirm knows what to do with the result
    model.select_context = Some(SelectContext::DeleteBranch);

    // Determine the preferred option from the commit under the cursor
    // For delete, only suggest actual branches (not revisions)
    let preferred = model
        .ui_model
        .lines
        .get(model.ui_model.cursor_position)
        .and_then(|line| {
            suggestions_from_line(line)
                .into_iter()
                .find(|s| matches!(s, BranchSuggestion::RemoteBranch(_)))
        });

    // Move the preferred option to the top
    if let Some(ref preferred) = preferred {
        let name = preferred.name();
        if let Some(idx) = branches.iter().position(|b| b == name) {
            let branch = branches.remove(idx);
            branches.insert(0, branch);
        }
    }

    // Show the select popup
    let state = SelectPopupState::new("Delete branch".to_string(), branches);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Select(state)));

    None
}
