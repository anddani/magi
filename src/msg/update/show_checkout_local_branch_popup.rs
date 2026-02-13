use crate::{
    git::checkout::{get_last_checked_out_branch, get_local_branches},
    model::{
        BranchSuggestion, Model,
        popup::{PopupContent, PopupContentCommand, SelectPopupState},
        select_popup::SelectContext,
        suggestions_from_line,
    },
    msg::Message,
};

pub fn update(model: &mut Model) -> Option<Message> {
    // Get only local branches, excluding the currently checked out branch
    let current_branch = model.git_info.current_branch();
    let mut branches: Vec<String> = get_local_branches(&model.git_info.repository)
        .into_iter()
        .filter(|b| current_branch.as_deref() != Some(b.as_str()))
        .collect();

    if branches.is_empty() {
        model.popup = Some(PopupContent::Error {
            message: "No local branches found".to_string(),
        });
        return None;
    }

    // Set the context so select_confirm knows what to do with the result
    model.select_context = Some(SelectContext::CheckoutBranch);

    // Determine the preferred option from the commit under the cursor
    // Only consider local branches for the suggestion
    let preferred = model
        .ui_model
        .lines
        .get(model.ui_model.cursor_position)
        .and_then(|line| {
            let suggestions = suggestions_from_line(line);
            suggestions.into_iter().find(|s| {
                matches!(s, BranchSuggestion::LocalBranch(name)
                    if current_branch.as_deref() != Some(name.as_str()))
            })
        })
        .or_else(|| {
            get_last_checked_out_branch(&model.git_info.repository)
                .filter(|b| branches.contains(b))
                .map(BranchSuggestion::LocalBranch)
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
    let state = SelectPopupState::new("Checkout local".to_string(), branches);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Select(state)));

    None
}
