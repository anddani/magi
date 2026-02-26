use crate::model::arguments::Argument;
use crate::model::popup::PopupContent;

pub mod update;
pub mod util;

/// Type of fixup-style commit (fixup or squash)
#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum FixupType {
    /// Create a fixup commit (git commit --fixup)
    Fixup,
    /// Create a squash commit (git commit --squash)
    Squash,
    /// Create an alter commit (git commit --fixup=amend: --edit)
    Alter,
    /// Create an augment commit (git commit --squash= --edit)
    Augment,
}

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
    /// Create a fixup or squash commit for the specified commit hash
    FixupCommit(String, FixupType),

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
    /// Actually pop stash after user confirms
    ConfirmPopStash(String),
    /// Actually drop stash after user confirms (stash ref or "all")
    ConfirmDropStash(String),

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
    /// Show the input popup for the new branch name (renaming old_name)
    ShowRenameBranchInput(String),
    /// Rename a branch
    RenameBranch {
        old_name: String,
        new_name: String,
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
    /// Show input popup for stash message, carrying which kind of stash to create
    ShowStashInput(StashType),

    Fetch(FetchCommand),
    Pull(PullCommand),
    Push(PushCommand),
    Stash(StashCommand),
    Rebase(RebaseCommand),

    /// Show rebase popup
    ShowRebasePopup,

    /// Show a select popup
    ShowSelectPopup(SelectPopup),

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

/// Messages for pull commands
#[derive(PartialEq, Eq, Debug)]
pub enum FetchCommand {
    // TODO: Should be combined with below
    /// Fetch from upstream
    FetchUpstream,
    // TODO: Rename to FetchFromRef
    /// Fetch from a specific remote/branch
    FetchFromRemoteBranch(String),
    /// Fetch from push remote (setting branch.<name>.pushRemote)
    FetchFromPushRemote(String),
    /// Fetch all remotes
    FetchAllRemotes,
    // TODO: Rename to FetchSubmodules
    /// Fetch all populated submodules (git fetch --recurse-submodules)
    FetchModules,
}

/// Messages for pull commands
#[derive(PartialEq, Eq, Debug)]
pub enum PullCommand {
    /// Pull from push remote (setting branch.<name>.pushRemote)
    PullFromPushRemote(String),
    /// Pull from a specific remote/branch (setting it as upstream)
    PullFromUpstream(String),
    /// Pull from upstream
    PullUpstream,
}

/// Messages for push commands
#[derive(PartialEq, Eq, Debug)]
pub enum PushCommand {
    /// Push to upstream (or create it if specified)
    PushUpstream,
    /// Push to a specific remote/branch (setting it as upstream)
    PushToRemote(String),
    /// Push to push remote (setting branch.<name>.pushRemote)
    PushToPushRemote(String),
    /// Push all tags to a specific remote
    PushAllTags(String),
    /// Push a single tag to origin
    PushTag(String),
}

/// Messages for rebase commands
#[derive(PartialEq, Eq, Debug)]
pub enum RebaseCommand {
    /// Rebase the current branch onto the given target ref/commit
    Elsewhere(String),
}

/// Which working-tree area to stash
#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum StashType {
    /// Stash both index and working tree (git stash push)
    Both,
    /// Stash only the index / staged changes (git stash push --staged)
    Index,
    /// Stash only the working tree, keeping the index intact (git stash push --keep-index)
    Workspace,
}

impl StashType {
    /// Human-readable title used in the input popup
    pub fn title(self) -> &'static str {
        match self {
            StashType::Both => "Stash message",
            StashType::Index => "Stash index message",
            StashType::Workspace => "Stash workspace message",
        }
    }

    /// Extra git flag for this stash type, if any
    pub fn flag(self) -> Option<&'static str> {
        match self {
            StashType::Both => None,
            StashType::Index => Some("--staged"),
            StashType::Workspace => Some("--keep-index"),
        }
    }

    /// Title shown in the PTY output panel
    pub fn pty_title(self) -> &'static str {
        match self {
            StashType::Both => "Stash",
            StashType::Index => "Stash index",
            StashType::Workspace => "Stash workspace",
        }
    }
}

/// Messages for stash commands
#[derive(PartialEq, Eq, Debug)]
pub enum StashCommand {
    /// Stash changes according to the given type and optional message
    Push(StashType, String),
    /// Apply a stash by its reference (e.g. "stash@{0}")
    Apply(String),
    /// Pop a stash by its reference (e.g. "stash@{0}") - applies and removes it
    Pop(String),
    /// Drop a stash by its reference (e.g. "stash@{0}"), or "all" to drop all stashes
    Drop(String),
}

/// Messages for showing select popups
#[derive(PartialEq, Eq, Debug)]
pub enum SelectPopup {
    // Fetch-related
    /// Show select popup to choose upstream for fetch
    FetchUpstream,
    /// Show select popup to choose a remote to fetch from
    FetchElsewhere,
    /// Show select popup to fetch a specific branch (picks remote first if multiple)
    FetchAnotherBranch,
    /// Show select popup to choose a branch from the given remote to fetch
    FetchAnotherBranchBranch(String),
    /// Show select popup to choose push remote for fetch
    FetchPushRemote,

    // Push-related
    /// Show select popup to choose upstream for push
    PushUpstream,
    /// Show select popup to choose push remote for push
    PushPushRemote,
    /// Show select popup to choose remote for pushing all tags
    PushAllTags,
    /// Show select popup to choose a tag to push
    PushTag,

    // Pull-related
    /// Show select popup to choose upstream for pull
    PullUpstream,
    /// Show select popup to choose push remote for pull
    PullPushRemote,

    // Branch-related
    /// Show the checkout branch select popup
    CheckoutBranch,
    /// Show the checkout local branch select popup (only local branches)
    CheckoutLocalBranch,
    /// Show the delete branch select popup
    DeleteBranch,
    /// Show the rename branch select popup (select branch to rename)
    RenameBranch,
    /// Show the create new branch popup (select starting point)
    CreateNewBranch { checkout: bool },

    // Stash-related
    /// Show apply stash popup, or immediately apply if cursor is on a stash entry
    StashApply,
    /// Show pop stash popup, or immediately pop if cursor is on a stash entry
    StashPop,
    /// Show drop stash popup, or immediately drop if cursor is on a stash entry
    StashDrop,

    // Fixup-related
    /// Show select popup to choose commit for fixup or squash
    FixupCommit(FixupType),

    // Rebase-related
    /// Show select popup (or confirm) to pick a commit/ref to rebase onto
    RebaseElsewhere,

    // PR-related
    /// Show select popup to pick source branch for PR (opens to default target)
    OpenPr,
    /// Show select popup to pick source branch for PR, then pick target
    OpenPrWithTarget,
    /// Show select popup to pick target branch for PR (source already chosen)
    OpenPrTarget(String),
}

/// Messages for the select popup
#[derive(PartialEq, Eq, Debug)]
pub enum SelectMessage {
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
