pub use super::select_popup::{SelectContext, SelectPopupState, SelectResult};

use crate::git::credential::CredentialType;

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
    CheckoutNewBranch {
        /// The starting point (branch, tag, or commit hash)
        starting_point: String,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PopupContentCommand {
    Commit,
    Push(PushPopupState),
    Fetch(FetchPopupState),
    Pull(PullPopupState),
    Branch,
    Log,
    Select(SelectPopupState),
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
