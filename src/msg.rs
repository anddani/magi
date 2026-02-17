use crate::model::arguments::Argument;
use crate::model::popup::PopupContent;

pub mod update;
pub mod util;

#[derive(PartialEq, Eq, Debug)]
pub enum Message {
    /// Quit application
    Quit,
    /// Refresh the buffer
    Refresh,
    /// Move one line up
    MoveUp,
    /// Move one line down
    MoveDown,
    /// Move half a page up
    HalfPageUp,
    /// Move half a page down
    HalfPageDown,
    /// Scroll viewport down by one line
    ScrollLineDown,
    /// Scroll viewport up by one line
    ScrollLineUp,
    /// Move cursor to the first visible line
    MoveToTop,
    /// Move cursor to the last visible line
    MoveToBottom,
    /// First 'g' press — waiting for second 'g' to trigger MoveToTop
    PendingG,
    /// Toggle section expand/collapse
    ToggleSection,

    /// Open commit in user's default EDITOR
    Commit,
    /// Amend the last commit
    Amend(Vec<String>),

    /// Dismiss the current popup
    DismissPopup,
    /// Stage all modified files (does not include untracked files)
    StageAllModified,
    /// Stage the item under the cursor (or visual selection)
    StageSelected,
    /// Unstage all staged files
    UnstageAll,

    /// Enter visual selection mode (sets anchor at current cursor position)
    EnterVisualMode,
    /// Exit visual selection mode (clears the anchor)
    ExitVisualMode,

    /// Show a popup with the given content (help, commit, branch, log)
    ShowPopup(PopupContent),
    /// Show push popup with options
    ShowPushPopup,
    /// Show fetch popup with options
    ShowFetchPopup,
    /// Show pull popup with options
    ShowPullPopup,
    /// Show the checkout branch select popup
    ShowCheckoutBranchPopup,
    /// Show the checkout local branch select popup (only local branches)
    ShowCheckoutLocalBranchPopup,
    /// Show the delete branch select popup
    ShowDeleteBranchPopup,
    /// Show the rename branch select popup (select branch to rename)
    ShowRenameBranchPopup,
    /// Show the input popup for the new branch name (renaming old_name)
    ShowRenameBranchInput(String),
    /// Rename a branch
    RenameBranch {
        old_name: String,
        new_name: String,
    },
    /// Show the create new branch popup (select starting point)
    ShowCreateNewBranchPopup {
        checkout: bool,
    },
    /// Show the input popup for new branch name
    ShowCreateNewBranchInput {
        starting_point: String,
        checkout: bool,
    },
    /// Create a new branch (optionally checkout)
    CreateNewBranch {
        starting_point: String,
        branch_name: String,
        checkout: bool,
    },
    /// Checkout the selected branch
    CheckoutBranch(String),
    /// Show confirmation popup before deleting the selected branch
    DeleteBranch(String),
    /// Actually delete the branch after user confirmation
    ConfirmDeleteBranch(String),
    /// Fetch all remotes
    FetchAllRemotes,
    /// Fetch from upstream
    FetchUpstream,
    /// Show select popup to choose upstream for fetch
    ShowFetchUpstreamSelect,
    /// Show select popup to choose a remote to fetch from
    ShowFetchElsewhereSelect,
    /// Fetch from a specific remote/branch
    FetchFromRemote(String),
    /// Push to upstream (or create it if specified)
    PushUpstream,
    /// Show select popup to choose upstream for push
    ShowPushUpstreamSelect,
    /// Push to a specific remote/branch (setting it as upstream)
    PushToRemote(String),
    /// Show select popup to choose remote for pushing all tags
    ShowPushAllTagsSelect,
    /// Push all tags to a specific remote
    PushAllTags(String),
    /// Show select popup to choose a tag to push
    ShowPushTagSelect,
    /// Push a single tag to origin
    PushTag(String),

    /// Pull from upstream
    PullUpstream,
    /// Show select popup to choose upstream for pull
    ShowPullUpstreamSelect,
    /// Pull from a specific remote/branch (setting it as upstream)
    PullFromRemote(String),

    /// Show select popup to pick source branch for PR (opens to default target)
    ShowOpenPrSelect,
    /// Show select popup to pick source branch for PR, then pick target
    ShowOpenPrWithTargetSelect,
    /// Show select popup to pick target branch for PR (source already chosen)
    ShowOpenPrTargetSelect(String),
    /// Open PR creation page in browser
    OpenPr {
        branch: String,
        target: Option<String>,
    },

    EnterArgMode,
    ToggleArgument(Argument),
    ExitArgMode,

    /// Select popup messages
    Select(SelectMessage),

    /// Input popup messages
    Input(InputMessage),

    /// Credentials popup
    Credentials(CredentialsMessage),

    /// Show the log view for the current branch
    ShowLogCurrent,
    /// Exit log view and return to status view
    ExitLogView,
}

/// Messages for the select popup
#[derive(PartialEq, Eq, Debug)]
pub enum SelectMessage {
    /// Show a select popup with the given title and options
    Show { title: String, options: Vec<String> },
    /// Input a character into select popup filter
    InputChar(char),
    /// Delete last character from select popup filter
    InputBackspace,
    /// Move selection up in select popup
    MoveUp,
    /// Move selection down in select popup
    MoveDown,
    /// Confirm selection in select popup
    Confirm,
}

/// Messages for text input popups (e.g., branch name input)
#[derive(PartialEq, Eq, Debug)]
pub enum InputMessage {
    /// Input a character into the text field
    InputChar(char),
    /// Delete last character from text field
    InputBackspace,
    /// Confirm the input
    Confirm,
}

#[derive(PartialEq, Eq, Debug)]
pub enum CredentialsMessage {
    /// Handle text input in credential popup
    CredentialInputChar(char),
    /// Handle backspace in credential popup input
    CredentialInputBackspace,
    /// Confirm credential input (submit the credential)
    CredentialConfirm,
}
