use crate::msg::{FixupType, ResetMode};

/// Result returned when select popup closes
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SelectResult {
    /// User selected an item
    Selected(String),
    /// User pressed Enter but no items matched the filter
    NoneSelected,
    /// User cancelled the popup (Esc)
    Cancelled,
}

/// What to do after the user picks an item (stored in popup state, replaces SelectContext)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OnSelect {
    /// Checkout the selected branch
    CheckoutBranch,
    /// Checkout a local-only branch (local branches only)
    CheckoutLocalBranch,
    /// Select a starting point for creating a new branch
    CreateNewBranchBase { checkout: bool },
    /// Select an upstream to push to
    PushUpstream,
    /// Select an upstream to fetch from
    FetchUpstream,
    /// Select a remote to fetch from (elsewhere)
    FetchElsewhere,
    /// Select an upstream to pull from
    PullUpstream,
    /// Select a branch to delete
    DeleteBranch,
    /// Select a branch to rename
    RenameBranch,
    /// Select a remote to push all tags to
    PushAllTags,
    /// Select a tag to push
    PushTag,
    /// Select a source branch for opening a PR (to default target)
    OpenPrBranch,
    /// Select a source branch for opening a PR, then pick target (step 1 of 2)
    OpenPrBranchWithTarget,
    /// Select a target branch for a PR (carries the source branch from step 1)
    OpenPrTarget { source_branch: String },
    /// Select a commit to fixup or squash
    FixupCommit(FixupType),
    /// Select a push remote to pull from (sets branch.<name>.pushRemote)
    PullPushRemote,
    /// Select a remote branch to pull from without changing any config (elsewhere)
    PullElsewhere,
    /// Select a remote branch to push to without changing any config (elsewhere)
    PushElsewhere,
    /// Select a push remote to push to (sets branch.<name>.pushRemote)
    PushPushRemote,
    /// Select a push remote to fetch from (sets branch.<name>.pushRemote)
    FetchPushRemote,
    /// Select a remote to fetch a specific branch from (step 1 of 2)
    FetchAnotherBranchRemote,
    /// Select a branch to fetch from the previously chosen remote (step 2 of 2)
    FetchAnotherBranch,
    /// Select a stash to apply
    ApplyStash,
    /// Select a stash to pop (apply and remove)
    PopStash,
    /// Select a stash to drop
    DropStash,
    /// Select a commit to rebase onto (rebase elsewhere)
    RebaseElsewhere,
    /// Select a local branch to reset (step 1 of 2)
    ResetBranchPick,
    /// Select a target to reset the given branch to (step 2 of 2, carries branch name)
    ResetBranchTarget { branch: String },
    /// Select a target to reset HEAD to
    Reset(ResetMode),
    /// Select a target for an index-only reset
    ResetIndex,
    /// Select a target for a worktree-only reset
    ResetWorktree,
    /// Select a branch/revision for a new worktree (checkout=true switches to it)
    WorktreeAdd { checkout: bool },
    /// Select a revision to checkout a file from (step 1 of 2)
    FileCheckoutRevision,
    /// Select a file to checkout (step 2 of 2, carries the chosen revision)
    FileCheckoutFile { revision: String },
    /// Select a local branch to push (step 1 of 2)
    PushOtherBranchPick,
    /// Select a remote branch to push to (step 2 of 2, carries the chosen local branch)
    PushOtherBranchTarget { local: String },
    /// Select a remote to push explicit refspecs to
    PushRefspecRemotePick,
    /// Select a remote to push matching branches to
    PushMatching,
    /// Select a remote to fetch explicit refspecs from
    FetchRefspecRemotePick,
    /// Select a commit to revise (reword)
    ReviseCommit,
    /// Select a branch to merge into the current branch
    MergeElsewhere,
    /// Select a commit to cherry-pick onto the current branch
    ApplyPick,
    /// Select a commit to apply (--no-commit) onto the current branch
    ApplyApply,
    /// Select a ref to squash-merge into the working tree (`git merge --squash`)
    ApplySquash,
    /// Select a commit to harvest (step 1 of 2, no pre-selection)
    HarvestCommitPick,
    /// Select a source branch to harvest commits from (has commits embedded)
    HarvestSourceBranch { commits: Vec<String> },
    /// Select a ref/commit to tag (carries tag name)
    CreateTagTarget { name: String },
    /// Select an existing tag to delete
    DeleteTag,
    /// Select a remote to prune tags against
    PruneTagsRemotePick,
    /// Select the mainline parent number when reverting a merge commit
    RevertMergeMainline {
        hashes: Vec<String>,
        no_commit: bool,
    },
}

/// Data source used to populate select popup options
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OptionsSource {
    /// Local branches only
    LocalBranches,
    /// Local branches + short remote branch names (get_branches)
    LocalAndRemoteBranches,
    /// All configured remotes
    Remotes,
    /// All branches from a specific remote (e.g. "origin/main", "origin/dev")
    RemoteBranches { remote: String },
    /// Remote branches formatted for upstream use (get_remote_branches_for_upstream)
    UpstreamBranches,
    /// Local tags only
    Tags,
    /// Local branches + local tags
    BranchesAndTags,
    /// Local branches (excluding already-checked-out) + local tags
    BranchesAndTagsExcludingCheckedOut,
    /// Local branches that have any configured remote
    LocalBranchesWithRemote,
    /// Local branches + remote branches + tags (for file checkout revision)
    FileCheckoutRevisions,
    /// All refs: local branches + remote branches + tags
    AllRefs,
    /// Stash entries (from UI model lines)
    Stashes,
    /// Tracked files (get_tracked_files)
    TrackedFiles,
}

/// State for the select popup (fuzzy finder style)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectPopupState {
    /// Display title (e.g., "Checkout", "Switch Branch")
    pub title: String,
    /// All available options (unfiltered)
    pub all_options: Vec<String>,
    /// Indices of options matching current filter
    pub filtered_indices: Vec<usize>,
    /// Current filter text
    pub input_text: String,
    /// Currently selected index in the filtered list (0-based)
    pub selected_index: usize,
    /// What to do when the user confirms a selection
    pub on_select: OnSelect,
}

impl SelectPopupState {
    /// Create a new select popup state with the given title, options, and action
    pub fn new(title: String, options: Vec<String>, on_select: OnSelect) -> Self {
        let filtered_indices: Vec<usize> = (0..options.len()).collect();
        Self {
            title,
            all_options: options,
            filtered_indices,
            input_text: String::new(),
            selected_index: 0,
            on_select,
        }
    }

    /// Returns the currently selected item, if any
    pub fn selected_item(&self) -> Option<&str> {
        self.filtered_indices
            .get(self.selected_index)
            .and_then(|&idx| self.all_options.get(idx))
            .map(|s| s.as_str())
    }

    /// Updates filtered_indices based on current input_text (case-insensitive substring)
    pub fn update_filter(&mut self) {
        let query = self.input_text.to_lowercase();
        self.filtered_indices = self
            .all_options
            .iter()
            .enumerate()
            .filter(|(_, opt)| opt.to_lowercase().contains(&query))
            .map(|(idx, _)| idx)
            .collect();
        // Reset selection to first match
        self.selected_index = 0;
    }

    /// Number of filtered hits
    pub fn filtered_count(&self) -> usize {
        self.filtered_indices.len()
    }

    /// Move selection up
    pub fn move_up(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }

    /// Move selection down
    pub fn move_down(&mut self) {
        if self.selected_index + 1 < self.filtered_indices.len() {
            self.selected_index += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dummy_on_select() -> OnSelect {
        OnSelect::CheckoutBranch
    }

    #[test]
    fn test_select_popup_state_new() {
        let options = vec![
            "main".to_string(),
            "feature".to_string(),
            "develop".to_string(),
        ];
        let state =
            SelectPopupState::new("Checkout".to_string(), options.clone(), dummy_on_select());

        assert_eq!(state.title, "Checkout");
        assert_eq!(state.all_options, options);
        assert_eq!(state.filtered_indices, vec![0, 1, 2]);
        assert_eq!(state.input_text, "");
        assert_eq!(state.selected_index, 0);
    }

    #[test]
    fn test_select_popup_state_selected_item() {
        let options = vec![
            "main".to_string(),
            "feature".to_string(),
            "develop".to_string(),
        ];
        let mut state = SelectPopupState::new("Checkout".to_string(), options, dummy_on_select());

        // Initially selects first item
        assert_eq!(state.selected_item(), Some("main"));

        // Move selection
        state.selected_index = 1;
        assert_eq!(state.selected_item(), Some("feature"));

        state.selected_index = 2;
        assert_eq!(state.selected_item(), Some("develop"));
    }

    #[test]
    fn test_select_popup_state_selected_item_empty() {
        let state = SelectPopupState::new("Checkout".to_string(), vec![], dummy_on_select());
        assert_eq!(state.selected_item(), None);
    }

    #[test]
    fn test_select_popup_state_update_filter() {
        let options = vec![
            "main".to_string(),
            "feature/auth".to_string(),
            "feature/ui".to_string(),
            "develop".to_string(),
        ];
        let mut state = SelectPopupState::new("Checkout".to_string(), options, dummy_on_select());

        // Filter for "feature"
        state.input_text = "feature".to_string();
        state.update_filter();
        assert_eq!(state.filtered_indices, vec![1, 2]);
        assert_eq!(state.selected_index, 0);
        assert_eq!(state.selected_item(), Some("feature/auth"));
    }

    #[test]
    fn test_select_popup_state_update_filter_case_insensitive() {
        let options = vec![
            "Main".to_string(),
            "FEATURE".to_string(),
            "develop".to_string(),
        ];
        let mut state = SelectPopupState::new("Checkout".to_string(), options, dummy_on_select());

        // Filter for "main" (lowercase) should match "Main"
        state.input_text = "main".to_string();
        state.update_filter();
        assert_eq!(state.filtered_indices, vec![0]);
        assert_eq!(state.selected_item(), Some("Main"));

        // Filter for "DEVELOP" (uppercase) should match "develop"
        state.input_text = "DEVELOP".to_string();
        state.update_filter();
        assert_eq!(state.filtered_indices, vec![2]);
        assert_eq!(state.selected_item(), Some("develop"));
    }

    #[test]
    fn test_select_popup_state_update_filter_no_matches() {
        let options = vec!["main".to_string(), "feature".to_string()];
        let mut state = SelectPopupState::new("Checkout".to_string(), options, dummy_on_select());

        state.input_text = "nonexistent".to_string();
        state.update_filter();
        assert!(state.filtered_indices.is_empty());
        assert_eq!(state.selected_item(), None);
    }

    #[test]
    fn test_select_popup_state_move_up() {
        let options = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        let mut state = SelectPopupState::new("Test".to_string(), options, dummy_on_select());

        state.selected_index = 2;
        state.move_up();
        assert_eq!(state.selected_index, 1);

        state.move_up();
        assert_eq!(state.selected_index, 0);

        // Should not go below 0
        state.move_up();
        assert_eq!(state.selected_index, 0);
    }

    #[test]
    fn test_select_popup_state_move_down() {
        let options = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        let mut state = SelectPopupState::new("Test".to_string(), options, dummy_on_select());

        assert_eq!(state.selected_index, 0);
        state.move_down();
        assert_eq!(state.selected_index, 1);

        state.move_down();
        assert_eq!(state.selected_index, 2);

        // Should not go beyond the last item
        state.move_down();
        assert_eq!(state.selected_index, 2);
    }

    #[test]
    fn test_select_popup_state_filtered_count() {
        let options = vec![
            "main".to_string(),
            "feature/a".to_string(),
            "feature/b".to_string(),
        ];
        let mut state = SelectPopupState::new("Test".to_string(), options, dummy_on_select());

        assert_eq!(state.filtered_count(), 3);

        state.input_text = "feature".to_string();
        state.update_filter();
        assert_eq!(state.filtered_count(), 2);

        state.input_text = "xyz".to_string();
        state.update_filter();
        assert_eq!(state.filtered_count(), 0);
    }

    #[test]
    fn test_select_popup_state_filter_resets_selection() {
        let options = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        let mut state = SelectPopupState::new("Test".to_string(), options, dummy_on_select());

        state.selected_index = 2;
        state.input_text = "b".to_string();
        state.update_filter();

        // Selection should reset to 0 after filter
        assert_eq!(state.selected_index, 0);
    }
}
