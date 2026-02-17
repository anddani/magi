use std::time::{Duration, Instant};

use crate::{
    git::{checkout::get_local_branches, open_pr::has_upstream},
    model::{
        BranchSuggestion, Model, Toast, ToastStyle,
        popup::{PopupContent, PopupContentCommand, SelectPopupState},
        select_popup::SelectContext,
        suggestions_from_line,
    },
    msg::Message,
};

const TOAST_DURATION: Duration = Duration::from_secs(5);

/// Shows a select popup with local branches that have an upstream set.
/// If `with_target` is true, uses `OpenPrBranchWithTarget` context (will ask for target next).
/// Otherwise uses `OpenPrBranch` context (opens PR to default target).
pub fn update(model: &mut Model, with_target: bool) -> Option<Message> {
    model.popup = None;

    let current_branch = match model.git_info.current_branch() {
        Some(branch) => branch,
        None => {
            model.toast = Some(Toast {
                message: "Not checked out to a branch (detached HEAD)".to_string(),
                style: ToastStyle::Warning,
                expires_at: Instant::now() + TOAST_DURATION,
            });
            return None;
        }
    };

    // Get only local branches that have an upstream set
    let mut branches: Vec<String> = get_local_branches(&model.git_info.repository)
        .into_iter()
        .filter(|b| has_upstream(&model.workdir, b))
        .collect();

    if branches.is_empty() {
        model.popup = Some(PopupContent::Error {
            message: "No branches with upstream found".to_string(),
        });
        return None;
    }

    // Determine the preferred branch: cursor branch first, then current branch
    let preferred = model
        .ui_model
        .lines
        .get(model.ui_model.cursor_position)
        .and_then(|line| {
            let suggestions = suggestions_from_line(line);
            suggestions.into_iter().find(|s| {
                matches!(s, BranchSuggestion::LocalBranch(name)
                    if branches.contains(name))
            })
        });

    let preferred_name = preferred
        .map(|s| s.name().to_string())
        .unwrap_or(current_branch);

    // Move the preferred option to the top
    if let Some(idx) = branches.iter().position(|b| b == &preferred_name) {
        let branch = branches.remove(idx);
        branches.insert(0, branch);
    }

    model.select_context = Some(if with_target {
        SelectContext::OpenPrBranchWithTarget
    } else {
        SelectContext::OpenPrBranch
    });

    let state = SelectPopupState::new("Open PR".to_string(), branches);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Select(state)));

    None
}
