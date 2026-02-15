use crate::model::arguments::Argument;

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
    Amend,
    /// Reword the last commit message (amend without staged changes)
    Reword,

    /// Dismiss the current popup
    DismissPopup,
    /// Stage all modified files (does not include untracked files)
    StageAllModified,
    /// Unstage all staged files
    UnstageAll,

    /// Enter visual selection mode (sets anchor at current cursor position)
    EnterVisualMode,
    /// Exit visual selection mode (clears the anchor)
    ExitVisualMode,

    /// Show help popup with keybindings
    ShowHelp,
    /// Show commit popup with options
    ShowCommitPopup,
    /// Show push popup with options
    ShowPushPopup,
    /// Show fetch popup with options
    ShowFetchPopup,
    /// Show pull popup with options
    ShowPullPopup,
    /// Show branch popup with options
    ShowBranchPopup,
    /// Show the checkout branch select popup
    ShowCheckoutBranchPopup,
    /// Show the checkout local branch select popup (only local branches)
    ShowCheckoutLocalBranchPopup,
    /// Show the delete branch select popup
    ShowDeleteBranchPopup,
    /// Show the checkout new branch popup (select starting point)
    ShowCheckoutNewBranchPopup,
    /// Show the input popup for new branch name (starting_point)
    ShowCheckoutNewBranchInput(String),
    /// Create and checkout a new branch (starting_point, new_branch_name)
    CheckoutNewBranch {
        starting_point: String,
        branch_name: String,
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

    EnterArgMode,
    ToggleArgument(Argument),
    ExitArgMode,

    /// Select popup messages
    Select(SelectMessage),

    /// Input popup messages
    Input(InputMessage),

    /// Credentials popup
    Credentials(CredentialsMessage),
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
