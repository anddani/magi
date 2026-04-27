use crate::i18n;
use crate::model::arguments::Argument;
use crate::model::popup::PopupContent;
pub use crate::model::select_popup::{OnSelect, OptionsSource};

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

/// Mode for `git reset` — controls how far the reset goes
#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum ResetMode {
    /// Reset HEAD and index, keep working tree (`git reset --mixed`)
    Mixed,
    /// Reset HEAD, index, and working tree (`git reset --hard`)
    Hard,
    /// Reset HEAD only (`git reset --soft`)
    Soft,
    /// Reset HEAD and index, keeping uncommitted (`git reset --keep`)
    Keep,
}

impl ResetMode {
    /// Name of reset mode
    pub fn name(self) -> &'static str {
        match self {
            ResetMode::Mixed => "Mixed",
            ResetMode::Hard => "Hard",
            ResetMode::Soft => "Soft",
            ResetMode::Keep => "Keep",
        }
    }

    /// The corresponding git flag
    pub fn flag(self) -> &'static str {
        match self {
            ResetMode::Mixed => "--mixed",
            ResetMode::Hard => "--hard",
            ResetMode::Soft => "--soft",
            ResetMode::Keep => "--keep",
        }
    }
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
    /// Navigate in the current buffer
    Navigation(NavigationAction),
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
    /// Show the input popup for the new spin-off branch name
    ShowSpinoffBranchInput,
    /// Create a spin-off branch from current HEAD (and reset current branch to upstream merge-base)
    SpinoffBranch(String),
    /// Show the input popup for the new spin-out branch name
    ShowSpinoutBranchInput,
    /// Create a spin-out branch from current HEAD (stays on current branch, resets it to upstream merge-base)
    SpinoutBranch(String),
    /// Checkout the selected branch
    CheckoutBranch(String),
    /// Show the input popup for the new worktree path
    ShowWorktreePathInput {
        branch: String,
        /// Whether to switch to the new worktree after creating it
        checkout: bool,
    },
    /// Add a new worktree at the given path, checking out branch
    WorktreeCheckout {
        branch: String,
        path: String,
        /// Whether to switch to the new worktree after creating it
        checkout: bool,
    },
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

    /// Show revert popup
    ShowRevertPopup,
    /// Show input popup for entering the -m mainline value for a revert
    ShowRevertMainlineInput,
    /// Execute a revert command
    Revert(RevertCommand),

    /// Show apply (cherry-pick) popup
    ShowApplyPopup,
    /// Execute an apply command
    Apply(ApplyCommand),
    /// Harvest commits from another branch (cherry-pick + remove from source)
    Harvest {
        commits: Vec<String>,
        source: String,
    },
    /// Donate commits to another existing branch (cherry-pick + remove from current)
    Donate {
        commits: Vec<String>,
        target: String,
    },
    /// Show input popup to name the new spin-out branch (carries commits and root start point)
    ShowCherrySpinoutInput {
        commits: Vec<String>,
        root: String,
    },
    /// Spin out commits to a new branch (cherry-pick + remove from current, stay on current)
    CherrySpinout {
        commits: Vec<String>,
        branch: String,
        root: String,
    },

    /// Show merge popup
    ShowMergePopup,
    /// Show tag popup
    ShowTagPopup,
    /// Show the input popup for entering a new tag name
    ShowCreateTagInput,
    /// Create a new tag pointing at the given target ref/commit
    CreateTag {
        name: String,
        target: String,
    },
    /// Delete an existing tag by name
    DeleteTag(String),
    /// Show confirmation popup for tag prune after computing local/remote diff
    ShowPruneTagsConfirm {
        remote: String,
    },
    /// Delete local-only tags and push deletions for remote-only tags
    PruneTags {
        /// Tags to delete locally (`git tag -d`)
        local_tags: Vec<String>,
        /// Tags to delete from the remote (`git push <remote> :tag`)
        remote_tags: Vec<String>,
        remote: String,
    },
    /// Execute a merge command
    Merge(MergeCommand),

    /// Show reset popup
    ShowResetPopup,
    /// Reset a branch to a target ref/commit using the given mode
    ResetBranch {
        branch: String,
        target: String,
        mode: ResetMode,
    },
    /// Reset the index to match a given tree-ish without touching HEAD or the working tree
    /// (equivalent to `git reset <target> -- .`)
    ResetIndex {
        target: String,
    },
    /// Reset the working tree to match a given tree-ish without touching HEAD or the index
    /// (equivalent to `git checkout <target> -- .`)
    ResetWorktree {
        target: String,
    },
    /// Checkout a single file from a given revision
    FileCheckout {
        revision: String,
        file: String,
    },

    /// Show a select popup
    ShowSelectPopup(ShowSelectPopupConfig),
    /// Show the input popup for entering refspec(s) to push to the given remote
    ShowPushRefspecInput(String),
    /// Show the input popup for entering refspec(s) to fetch from the given remote
    ShowFetchRefspecInput(String),

    /// Show a commit select view (Log view)
    ShowCommitSelect(CommitSelect),

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

    /// Enter search input mode (press '/')
    EnterSearchMode,
    /// Search messages (input, navigate, cancel)
    Search(SearchMessage),

    /// Enter preview mode for the commit/stash under cursor
    ShowPreview,
    /// Exit preview mode and return to previous view
    ExitPreview,

    /// Revise (reword) a specific commit via `git commit --fixup=reword:<hash> --edit`
    ReviseCommit(String),
}

#[derive(PartialEq, Eq, Debug)]
pub enum NavigationAction {
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
}

/// Messages for search mode
#[derive(PartialEq, Eq, Debug)]
pub enum SearchMessage {
    /// Input a character into the search field
    InputChar(char),
    /// Delete last character from the search field
    InputBackspace,
    /// Confirm the search query (press Enter)
    Confirm,
    /// Go to next match
    Next,
    /// Go to previous match
    Prev,
    /// Cancel/clear the search
    Cancel,
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
    /// Fetch explicit refspecs from a remote without changing any config
    FetchRefspecs { remote: String, refspecs: String },
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
    /// Pull from a remote branch without modifying any git config (elsewhere)
    PullFromElsewhere(String),
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
    /// Push to a remote branch without modifying any git config (elsewhere)
    PushElsewhere(String),
    /// Push a specific local branch to a remote branch without changing any config
    PushOtherBranch { local: String, remote: String },
    /// Push explicit refspecs to a remote without changing any config
    PushRefspecs { remote: String, refspecs: String },
    /// Push matching branches (same-named on remote) using refspec `:`
    PushMatching(String),
}

/// Messages for rebase commands
#[derive(PartialEq, Eq, Debug)]
pub enum RebaseCommand {
    /// Rebase the current branch onto the given target ref/commit
    Elsewhere(String),
    /// Continue after resolving conflicts
    Continue,
    /// Skip the current conflicting commit
    Skip,
    /// Abort the rebase sequence
    Abort,
}

/// Messages for merge commands
#[derive(PartialEq, Eq, Debug)]
pub enum MergeCommand {
    /// Merge the given branch into the current branch
    Branch(String),
    /// Continue after resolving conflicts
    Continue,
    /// Abort the merge sequence
    Abort,
}

/// Messages for apply (cherry-pick) commands
#[derive(PartialEq, Eq, Debug)]
pub enum ApplyCommand {
    /// Cherry-pick the given commit hashes (creates new commits)
    Pick(Vec<String>),
    /// Apply the given commit hashes without committing (`git cherry-pick --no-commit`)
    Apply(Vec<String>),
    /// Squash the given ref into the working tree (`git merge --squash`)
    Squash(String),
    /// Continue after resolving conflicts
    Continue,
    /// Skip the current conflicting commit
    Skip,
    /// Abort the cherry-pick sequence
    Abort,
}

/// Messages for revert commands
#[derive(PartialEq, Eq, Debug)]
pub enum RevertCommand {
    /// Revert the given commit hashes, creating a revert commit (--no-edit)
    Commits {
        hashes: Vec<String>,
        mainline: Option<String>,
    },
    /// Revert the given commit hashes into the worktree without committing (--no-commit)
    NoCommit {
        hashes: Vec<String>,
        mainline: Option<String>,
    },
    /// Revert merge commit(s) with an explicit mainline parent number (-m)
    CommitsWithMainline {
        hashes: Vec<String>,
        mainline: u8,
        no_commit: bool,
    },
    /// Continue after resolving conflicts
    Continue,
    /// Skip the current conflicting commit
    Skip,
    /// Abort the revert sequence
    Abort,
}

/// Which working-tree area to stash
#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum StashType {
    /// Stash both index and working tree (git stash push)
    Both,
    /// Stash only the index / staged changes (git stash push --staged)
    Index,
    /// Stash only the working tree, keeping the index intact (git stash push --keep-index)
    Worktree,
}

impl StashType {
    /// Human-readable title used in the input popup
    pub fn title(self) -> &'static str {
        let t = i18n::t();
        match self {
            StashType::Both => t.input_stash_message,
            StashType::Index => t.input_stash_index_message,
            StashType::Worktree => t.input_stash_worktree_message,
        }
    }

    /// Extra git flag for this stash type, if any
    pub fn flag(self) -> Option<&'static str> {
        match self {
            StashType::Both => None,
            StashType::Index => Some("--staged"),
            StashType::Worktree => Some("--keep-index"),
        }
    }

    /// Title shown in the PTY output panel
    pub fn pty_title(self) -> &'static str {
        match self {
            StashType::Both => "Stash",
            StashType::Index => "Stash index",
            StashType::Worktree => "Stash worktree",
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

/// Messages for commit select
#[derive(PartialEq, Eq, Debug)]
pub enum CommitSelect {
    // Fixup-related
    /// Show select popup to choose commit for fixup or squash
    FixupCommit(FixupType),

    // Rebase-related
    /// Show select popup (or confirm) to pick a commit/ref to rebase onto
    RebaseElsewhere,

    // Revise-related
    /// Show select popup (or confirm) to pick a commit to revise (reword)
    ReviseCommit,
}
/// Config for showing a select popup (replaces the old SelectPopup enum)
#[derive(PartialEq, Eq, Debug)]
pub struct ShowSelectPopupConfig {
    /// The title displayed at the top of the popup
    pub title: String,
    /// Where to fetch the list of options from
    pub source: OptionsSource,
    /// What to do when the user confirms a selection
    pub on_select: OnSelect,
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
