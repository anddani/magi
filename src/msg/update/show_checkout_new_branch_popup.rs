use crate::{
    git::{checkout::get_branches, push::get_local_tags},
    model::{
        BranchSuggestion, Model,
        popup::{PopupContent, PopupContentCommand, SelectPopupState},
        select_popup::SelectContext,
        suggestions_from_line,
    },
    msg::Message,
};

pub fn update(model: &mut Model) -> Option<Message> {
    // Get all references: local branches first, then remote branches, then tags
    let branches = get_branches(&model.git_info.repository);
    let tags = get_local_tags(&model.git_info.repository);

    let mut options: Vec<String> = Vec::new();

    // Determine the preferred option from the line under the cursor,
    // falling back to the current branch if no specific commit/revision is highlighted
    let preferred = model
        .ui_model
        .lines
        .get(model.ui_model.cursor_position)
        .and_then(|line| {
            let suggestions = suggestions_from_line(line);
            suggestions.into_iter().next()
        })
        .or_else(|| {
            model
                .git_info
                .current_branch()
                .map(|b| BranchSuggestion::LocalBranch(b.to_string()))
        });

    // If there's a preferred suggestion, add it first
    if let Some(ref preferred) = preferred {
        options.push(preferred.name().to_string());
    }

    // Add branches (filtering out the preferred one if it was already added)
    for branch in branches {
        if preferred
            .as_ref()
            .map(|p| p.name() != branch)
            .unwrap_or(true)
        {
            options.push(branch);
        }
    }

    // Add tags (filtering out the preferred one if it was already added)
    for tag in tags {
        if preferred
            .as_ref()
            .map(|p| match p {
                BranchSuggestion::Revision(rev) => rev != &tag,
                _ => p.name() != tag,
            })
            .unwrap_or(true)
        {
            options.push(tag);
        }
    }

    if options.is_empty() {
        model.popup = Some(PopupContent::Error {
            message: "No references found".to_string(),
        });
        return None;
    }

    // Set the context so select_confirm knows what to do with the result
    model.select_context = Some(SelectContext::CheckoutNewBranchBase);

    // Show the select popup
    let state = SelectPopupState::new("Create branch starting at".to_string(), options);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Select(state)));

    None
}
