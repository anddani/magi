pub use super::select_popup::{SelectContext, SelectPopupState, SelectResult};

use crate::git::credential::CredentialType;

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
}

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
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PopupContentCommand {
    Commit,
    Push(PushPopupState),
    Fetch(FetchPopupState),
    Branch,
    Select(SelectPopupState),
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
