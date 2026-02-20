use crate::{
    git::checkout::get_local_branches,
    model::{
        Model,
        popup::{InputContext, InputPopupState, PopupContent},
    },
    msg::Message,
};

/// Suggests a local branch name from a remote branch name.
/// For example, "origin/feat/my-feat" -> Some("feat/my-feat")
/// Returns None if:
/// - The branch doesn't look like a remote branch (no '/' separator)
/// - The suggested local name already exists
fn suggest_local_branch_name(starting_point: &str, local_branches: &[String]) -> Option<String> {
    // Check if this looks like a remote branch (contains a '/')
    let (remote, local_part) = starting_point.split_once('/')?;

    // Common remote names - we only suggest for these to avoid false positives
    // with local branches that have '/' in their names
    if !["origin", "upstream"].contains(&remote) {
        return None;
    }

    // Check if the suggested local name already exists
    if local_branches.iter().any(|b| b == local_part) {
        return None;
    }

    Some(local_part.to_string())
}

pub fn update(model: &mut Model, starting_point: String, checkout: bool) -> Option<Message> {
    // Try to suggest a local branch name if creating from a remote branch
    let suggested_name = {
        let local_branches = get_local_branches(&model.git_info.repository);
        suggest_local_branch_name(&starting_point, &local_branches)
    };

    // Show the input popup for the new branch name
    let mut state = InputPopupState::new(
        "Name for new branch".to_string(),
        InputContext::CreateNewBranch {
            starting_point,
            checkout,
        },
    );

    // Pre-fill with suggested name if available
    if let Some(name) = suggested_name {
        state.input_text = name;
    }

    model.popup = Some(PopupContent::Input(state));

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_suggest_local_branch_name_from_origin() {
        let local_branches = vec!["main".to_string(), "develop".to_string()];

        // Should suggest local name for origin remote
        assert_eq!(
            suggest_local_branch_name("origin/feat/my-feat", &local_branches),
            Some("feat/my-feat".to_string())
        );

        assert_eq!(
            suggest_local_branch_name("origin/feature-x", &local_branches),
            Some("feature-x".to_string())
        );
    }

    #[test]
    fn test_suggest_local_branch_name_from_upstream() {
        let local_branches = vec!["main".to_string()];

        // Should suggest local name for upstream remote
        assert_eq!(
            suggest_local_branch_name("upstream/develop", &local_branches),
            Some("develop".to_string())
        );
    }

    #[test]
    fn test_suggest_local_branch_name_already_exists() {
        let local_branches = vec!["main".to_string(), "feat/my-feat".to_string()];

        // Should return None if the local branch already exists
        assert_eq!(
            suggest_local_branch_name("origin/feat/my-feat", &local_branches),
            None
        );
    }

    #[test]
    fn test_suggest_local_branch_name_not_remote() {
        let local_branches = vec!["main".to_string()];

        // Should return None for local branches (no '/')
        assert_eq!(suggest_local_branch_name("main", &local_branches), None);

        assert_eq!(
            suggest_local_branch_name("feature-x", &local_branches),
            None
        );
    }

    #[test]
    fn test_suggest_local_branch_name_unknown_remote() {
        let local_branches = vec!["main".to_string()];

        // Should return None for non-standard remote names to avoid false positives
        // with local branches that have '/' in their names
        assert_eq!(
            suggest_local_branch_name("custom-remote/feature", &local_branches),
            None
        );
    }

    #[test]
    fn test_suggest_local_branch_name_with_multiple_slashes() {
        let local_branches = vec!["main".to_string()];

        // Should handle remote branches with multiple slashes in the branch name
        assert_eq!(
            suggest_local_branch_name("origin/feat/sub/my-feat", &local_branches),
            Some("feat/sub/my-feat".to_string())
        );
    }
}
