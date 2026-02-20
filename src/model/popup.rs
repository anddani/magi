pub use super::select_popup::{SelectContext, SelectPopupState, SelectResult};

use crate::git::credential::CredentialType;
use crate::model::LogEntry;

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
}

/// State for the Fetch popup
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FetchPopupState {
    /// The current upstream branch name, if set
    pub upstream: Option<String>,
}

/// State for the Pull popup
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PullPopupState {
    /// The current upstream branch name, if set
    pub upstream: Option<String>,
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
}

/// State for text input popups (e.g., new branch name)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InputPopupState {
    /// Title displayed in the popup
    pub title: String,
    /// The text the user has entered so far
    pub input_text: String,
    /// Context for what action to perform when input is confirmed
    pub context: InputContext,
}

impl InputPopupState {
    /// Create a new input popup state
    pub fn new(title: String, context: InputContext) -> Self {
        Self {
            title,
            input_text: String::new(),
            context,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PopupContentCommand {
    Commit,
    Push(PushPopupState),
    Fetch(FetchPopupState),
    Pull(PullPopupState),
    Branch,
    Log,
    Select(SelectPopupState),
    CommitSelect(CommitSelectPopupState),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PopupContent {
    Error {
        message: String,
    },
    Help,
    Command(PopupContentCommand),
    /// Credential input popup for password/passphrase/etc.
    Credential(CredentialPopupState),
    /// Confirmation popup that requires y/Enter to confirm or n/Esc to cancel.
    /// The message field stores the associated data needed after confirmation.
    Confirm(ConfirmPopupState),
    /// Text input popup for entering arbitrary text (e.g., branch name)
    Input(InputPopupState),
}
