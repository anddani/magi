pub use super::select_popup::{OnSelect, OptionsSource, SelectPopupState, SelectResult};

use crate::git::credential::CredentialType;
use crate::i18n;
use crate::model::LogEntry;
use crate::msg::StashType;

/// State for a confirmation popup (e.g., "Are you sure you want to delete?")
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfirmPopupState {
    /// The question to display to the user
    pub message: String,
    /// The message to dispatch if the user confirms
    pub on_confirm: ConfirmAction,
}

/// Actions that can be triggered by a confirmation popup
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfirmAction {
    /// Delete a branch (stores the branch name)
    DeleteBranch(String),
    /// Discard changes (stores the discard target)
    DiscardChanges(crate::msg::DiscardTarget),
    /// Pop a stash (stores the stash reference, e.g. "stash@{0}")
    PopStash(String),
    /// Drop a stash (stores the stash reference, e.g. "stash@{0}" or "all")
    DropStash(String),
    /// Rebase the current branch onto the given target ref/commit
    RebaseElsewhere(String),
    /// Revise (reword) a commit via `git commit --fixup=reword:<hash> --edit`
    ReviseCommit(String),
    /// Reset a branch to a target ref/commit
    ResetBranch {
        branch: String,
        target: String,
        mode: crate::msg::ResetMode,
    },
    /// Prune tags: delete local-only tags and push-delete remote-only tags
    PruneTags {
        local_tags: Vec<String>,
        remote_tags: Vec<String>,
        remote: String,
    },
}

/// State for the credential input popup.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CredentialPopupState {
    /// The type of credential being requested.
    pub credential_type: CredentialType,
    /// The text the user has entered so far.
    pub input_text: String,
}

/// State for the Push popup
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PushPopupState {
    /// The current upstream branch name, if set
    pub upstream: Option<String>,
    /// The configured push remote name (branch.<name>.pushRemote), if set
    pub push_remote: Option<String>,
    /// The sole remote name, if exactly one remote is configured
    pub sole_remote: Option<String>,
}

/// State for the Fetch popup
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FetchPopupState {
    /// The current upstream branch name, if set
    pub upstream: Option<String>,
    /// The configured push remote name (branch.<name>.pushRemote), if set
    pub push_remote: Option<String>,
    /// The sole remote name, if exactly one remote is configured
    pub sole_remote: Option<String>,
}

/// State for the Pull popup
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PullPopupState {
    /// The current upstream branch name, if set
    pub upstream: Option<String>,
    /// The configured push remote name (branch.<name>.pushRemote), if set
    pub push_remote: Option<String>,
    /// The sole remote name, if exactly one remote is configured
    pub sole_remote: Option<String>,
}

/// Context for what action the input popup is performing
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InputContext {
    /// Creating a new branch from a starting point
    CreateNewBranch {
        /// The starting point (branch, tag, or commit hash)
        starting_point: String,
        /// Whether to checkout the branch after creation
        checkout: bool,
    },
    /// Renaming an existing branch
    RenameBranch {
        /// The current name of the branch being renamed
        old_name: String,
    },
    /// Stash push input — carries which kind of stash to create
    Stash(StashType),
    /// Creating a new spin-off branch from the current HEAD
    SpinoffBranch,
    /// Creating a new spin-out branch from the current HEAD (stays on current branch)
    SpinoutBranch,
    /// Entering the directory path for a new worktree
    WorktreePath {
        /// The branch or revision to check out in the new worktree
        branch: String,
        /// Whether to switch to the new worktree after creating it
        checkout: bool,
    },
    /// Entering refspec(s) to push to a remote (comma-separated)
    PushRefspec {
        /// The remote to push to
        remote: String,
    },
    /// Entering refspec(s) to fetch from a remote (comma-separated)
    FetchRefspec {
        /// The remote to fetch from
        remote: String,
    },
    /// Creating a new tag (name input; target picked next)
    CreateTag,
    /// Entering the mainline parent number for a revert of a merge commit
    RevertMainline { revert_state: RevertPopupState },
}

/// State for text input popups (e.g., new branch name)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InputPopupState {
    /// The text the user has entered so far
    pub input_text: String,
    /// Context for what action to perform when input is confirmed
    pub context: InputContext,
}

impl InputPopupState {
    /// Create a new input popup state
    pub fn new(context: InputContext) -> Self {
        Self {
            input_text: String::new(),
            context,
        }
    }

    pub fn title(&self) -> String {
        let t = i18n::t();
        match &self.context {
            InputContext::CreateNewBranch { .. } => t.input_new_branch.to_string(),
            InputContext::RenameBranch { old_name } => t.fmt1(t.input_rename_branch_fmt, old_name),
            InputContext::Stash(stash_type) => stash_type.title().to_string(),
            InputContext::SpinoffBranch => t.input_spinoff_branch.to_string(),
            InputContext::SpinoutBranch => t.input_spinout_branch.to_string(),
            InputContext::WorktreePath { branch, .. } => t.fmt1(t.input_worktree_path_fmt, branch),
            InputContext::PushRefspec { remote } => t.fmt1(t.input_push_refspec_fmt, remote),
            InputContext::FetchRefspec { remote } => t.fmt1(t.input_fetch_refspec_fmt, remote),
            InputContext::CreateTag => t.input_tag_name.to_string(),
            InputContext::RevertMainline { .. } => t.input_revert_mainline.to_string(),
        }
    }
}

/// State for commit select popup (shows commits with log formatting)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommitSelectPopupState {
    /// Display title (e.g., "Fixup commit", "Squash commit")
    pub title: String,
    /// All available commits (unfiltered)
    pub all_commits: Vec<LogEntry>,
    /// Indices of commits matching current filter
    pub filtered_indices: Vec<usize>,
    /// Current filter text
    pub input_text: String,
    /// Currently selected index in the filtered list (0-based)
    pub selected_index: usize,
}

impl CommitSelectPopupState {
    /// Create a new commit select popup state with the given title and commits
    pub fn new(title: String, commits: Vec<LogEntry>) -> Self {
        let filtered_indices: Vec<usize> = (0..commits.len()).collect();
        Self {
            title,
            all_commits: commits,
            filtered_indices,
            input_text: String::new(),
            selected_index: 0,
        }
    }

    /// Returns the currently selected commit hash, if any
    pub fn selected_commit_hash(&self) -> Option<&str> {
        self.filtered_indices
            .get(self.selected_index)
            .and_then(|&idx| self.all_commits.get(idx))
            .and_then(|entry| entry.hash.as_deref())
    }

    /// Returns the currently selected commit, if any
    pub fn selected_commit(&self) -> Option<&LogEntry> {
        self.filtered_indices
            .get(self.selected_index)
            .and_then(|&idx| self.all_commits.get(idx))
    }

    /// Updates filtered_indices based on current input_text (case-insensitive substring)
    /// Searches in hash, message, and author
    pub fn update_filter(&mut self) {
        let query = self.input_text.to_lowercase();
        self.filtered_indices = self
            .all_commits
            .iter()
            .enumerate()
            .filter(|(_, entry)| {
                entry
                    .hash
                    .as_ref()
                    .map(|h| h.to_lowercase().contains(&query))
                    .unwrap_or(false)
                    || entry
                        .message
                        .as_ref()
                        .map(|m| m.to_lowercase().contains(&query))
                        .unwrap_or(false)
                    || entry
                        .author
                        .as_ref()
                        .map(|a| a.to_lowercase().contains(&query))
                        .unwrap_or(false)
            })
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

/// State for the Rebase popup
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RebasePopupState {
    /// The currently checked out branch name
    pub branch: String,
    /// Whether a rebase sequence is currently in progress (conflict stopped)
    pub in_progress: bool,
}

/// State for the Revert popup
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RevertPopupState {
    /// Whether a revert sequence is currently in progress (conflict stopped)
    pub in_progress: bool,
    /// Commit hashes selected for reverting (empty when in_progress or no commit under cursor)
    pub selected_commits: Vec<String>,
    /// Mainline parent number set via `-m` argument (bypasses the select popup when set)
    pub mainline: Option<String>,
}

/// State for the Apply (cherry-pick) popup
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApplyPopupState {
    /// Whether a cherry-pick sequence is currently in progress (conflict stopped)
    pub in_progress: bool,
    /// Commit hashes selected for cherry-picking (empty when in_progress or no commit under cursor)
    pub selected_commits: Vec<String>,
}

/// State for the Merge popup
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MergePopupState {
    /// Whether a merge sequence is currently in progress (conflict stopped)
    pub in_progress: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PopupContentCommand {
    Commit,
    Push(PushPopupState),
    Fetch(FetchPopupState),
    Pull(PullPopupState),
    Branch,
    Log,
    Stash,
    Reset,
    Rebase(RebasePopupState),
    Revert(RevertPopupState),
    Merge(MergePopupState),
    Apply(ApplyPopupState),
    Tag,
    Select(SelectPopupState),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PopupContent {
    /// Shows an error popup
    Error { message: String },
    /// Shows the help popup
    Help,
    /// Shows a command popup
    Command(PopupContentCommand),
    /// Credential input popup for password/passphrase/etc.
    Credential(CredentialPopupState),
    /// Confirmation popup that requires y/Enter to confirm or n/Esc to cancel.
    /// The message field stores the associated data needed after confirmation.
    Confirm(ConfirmPopupState),
    /// Text input popup for entering arbitrary text (e.g., branch name)
    Input(InputPopupState),
}

impl PopupContent {
    pub fn input_popup(context: InputContext) -> Self {
        Self::Input(InputPopupState::new(context))
    }
    pub fn error(message: String) -> Self {
        Self::Error { message }
    }
}
