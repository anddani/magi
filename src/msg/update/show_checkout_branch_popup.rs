use crate::{
    git::{
        checkout::{get_branches, get_last_checked_out_branch},
        CommitRefType,
    },
    model::{
        popup::{PopupContent, PopupContentCommand, SelectPopupState},
        select_popup::SelectContext,
        LineContent, Model,
    },
    msg::Message,
};

pub fn update(model: &mut Model) -> Option<Message> {
    // Get all branches, excluding the currently checked out branch
    let current_branch = model
        .git_info
        .repository
        .head()
        .ok()
        .and_then(|head| head.shorthand().map(String::from));
    let mut branches: Vec<String> = get_branches(&model.git_info.repository)
        .into_iter()
        .filter(|b| current_branch.as_deref() != Some(b.as_str()))
        .collect();

    if branches.is_empty() {
        model.popup = Some(PopupContent::Error {
            message: "No branches found".to_string(),
        });
        return None;
    }

    // Set the context so select_confirm knows what to do with the result
    model.select_context = Some(SelectContext::CheckoutBranch);

    // Determine the preferred option from the commit under the cursor
    let preferred = model
        .ui_model
        .lines
        .get(model.ui_model.cursor_position)
        .and_then(|line| match &line.content {
            LineContent::Commit(commit_info) => {
                // Prefer a local branch, fall back to a remote branch
                // Skip the current branch since it's already checked out
                let local = commit_info.refs.iter().find(|r| {
                    r.ref_type == CommitRefType::LocalBranch
                        && current_branch.as_deref() != Some(r.name.as_str())
                });
                let remote = commit_info
                    .refs
                    .iter()
                    .find(|r| r.ref_type == CommitRefType::RemoteBranch);
                local
                    .or(remote)
                    .map(|r| r.name.clone())
                    .or_else(|| Some(commit_info.hash.clone()))
            }
            _ => None,
        })
        .or_else(|| get_last_checked_out_branch(&model.git_info.repository));

    // Move the preferred option to the top, or insert the commit hash
    if let Some(ref preferred) = preferred {
        if let Some(idx) = branches.iter().position(|b| b == preferred) {
            let branch = branches.remove(idx);
            branches.insert(0, branch);
        } else {
            // Hash not in the branch list — insert it at the top
            branches.insert(0, preferred.clone());
        }
    }

    // Show the select popup
    let state = SelectPopupState::new("Checkout".to_string(), branches);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Select(state)));

    None
}
