use crate::model::arguments::Argument;
use crate::model::popup::PopupContent;

pub mod update;
pub mod util;

/// Type of log view to display
#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum LogType {
    /// Show log for current branch (HEAD)
    Current,
    /// Show log for all references (--all)
    AllReferences,
    /// Show log for local branches and HEAD
    LocalBranches,
    /// Show log for local and remote branches and HEAD
    AllBranches,
}

/// Source of changes to discard
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiscardSource {
    /// Discard unstaged changes (working tree)
    Unstaged,
    /// Discard staged changes (index) - for new files, this deletes them
    Staged,
    /// Discard untracked files (deletes them from disk)
    Untracked,
}

/// Target for discard operations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiscardTarget {
    /// Discard entire files
    Files {
        paths: Vec<String>,
        source: DiscardSource,
    },
    /// Discard a single hunk
    Hunk {
        path: String,
        hunk_index: usize,
        source: DiscardSource,
    },
    /// Discard multiple hunks in the same file
    Hunks {
        path: String,
        hunk_indices: Vec<usize>,
        source: DiscardSource,
    },
    /// Discard specific lines within a hunk
    Lines {
        path: String,
        hunk_index: usize,
        line_indices: Vec<usize>,
        source: DiscardSource,
    },
}

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
    /// Show select popup to choose commit for fixup
    ShowFixupCommitSelect,
    /// Create a fixup commit for the specified commit hash
    FixupCommit(String),

    /// Dismiss the current popup
    DismissPopup,
    /// Stage all modified files (does not include untracked files)
    StageAllModified,
    /// Stage the item under the cursor (or visual selection)
    StageSelected,
    /// Unstage the item under the cursor (or visual selection)
    UnstageSelected,
    /// Unstage all staged files
    UnstageAll,

    /// Discard changes under cursor (shows confirmation popup)
    DiscardSelected,
    /// Actually discard after user confirms
    ConfirmDiscard(DiscardTarget),

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

    /// Show the log view
    ShowLog(LogType),
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
