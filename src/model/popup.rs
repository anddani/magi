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
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PopupContentCommand {
    Commit,
    Push(PushPopupState),
    Fetch,
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
