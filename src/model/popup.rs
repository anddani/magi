pub use super::select_popup::{SelectPopupState, SelectResult};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PopupContent {
    Error { message: String },
    Command(PopupContentCommand),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PopupContentCommand {
    Help,
    Commit,
    Push(PushPopupState),
    Select(SelectPopupState),
}

/// State for the Push popup
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PushPopupState {
    /// The local branch name (used as suggestion for new upstream)
    pub local_branch: String,
    /// The current upstream branch name, if set
    pub upstream: Option<String>,
    /// The default remote to use (first configured remote)
    pub default_remote: String,
    /// When true, user is entering a custom upstream branch name
    pub input_mode: bool,
    /// The text input for the remote/branch (e.g., "origin/feature-branch")
    pub input_text: String,
}
