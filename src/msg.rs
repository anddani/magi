use crate::i18n;
use crate::model::arguments::Argument;
use crate::model::input_field::EditOp;
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
#[derive(PartialEq, Eq, Debug, Clone)]
pub enum LogType {
    /// Show log for current branch (HEAD)
    Current,
    /// Show log for another branch/revision
    Other(String),
    /// Show log for the current branch, its upstream and its push target
    Related,
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
    /// Show the author picker for the commit `-A` argument, or clear the
    /// author override if one is already set
    ShowCommitAuthorSelect,
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
    /// Apply the change under the cursor (or visual selection) to the working
    /// tree. Only meaningful in Preview mode, where the shown diff comes from
    /// a commit or stash rather than the working tree.
    ApplySelected,

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
    /// Show log popup with default arguments
    ShowLogPopup,
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
    /// Open the interactive rebase todo editor for `base..HEAD` (base inclusive)
    ShowRebaseTodo(String),
    /// Interactive rebase todo editor messages
    RebaseTodo(RebaseTodoMessage),

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
    /// Show input popup to name the new spin-off branch (carries commits and root start point)
    ShowCherrySpinoffInput {
        commits: Vec<String>,
        root: String,
    },
    /// Spin off commits to a new branch (cherry-pick + remove from current, checkout new branch)
    CherrySpinoff {
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
    /// Runs `git tag --edit ...` which opens the user's configured editor
    /// for the tag message. Requires the TUI to be suspended.
    CreateTagWithEditor {
        name: String,
        args: Vec<String>,
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
    /// Apply a text editing operation to the search field
    Edit(EditOp),
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
    /// Rebase the current branch onto its push remote branch
    /// (sets `branch.<name>.pushRemote` to the given remote)
    OntoPushRemote(String),
    /// Rebase the current branch onto its configured upstream branch
    OntoUpstream,
    /// Rebase the current branch onto the given remote branch (setting it as upstream)
    OntoUpstreamSetting(String),
    /// Rebase the current branch onto the given target ref/commit
    Elsewhere(String),
    /// Rebase a subset of the current branch's history onto a new base:
    /// commits from `start` (inclusive) to HEAD are rebased onto `newbase`
    Subset { newbase: String, start: String },
    /// Run the interactive rebase using the todo list in `model.rebase_todo`
    ExecuteInteractive,
    /// Start an interactive rebase that stops at the given commit for editing
    /// (the commit is marked `edit`, everything after it is picked)
    ModifyCommit(String),
    /// Start an interactive rebase that rewords the given commit
    /// (the commit is marked `reword`, everything after it is picked)
    RewordCommit(String),
    /// Continue after resolving conflicts
    Continue,
    /// Skip the current conflicting commit
    Skip,
    /// Abort the rebase sequence
    Abort,
}

/// Messages for the interactive rebase todo editor
#[derive(PartialEq, Eq, Debug)]
pub enum RebaseTodoMessage {
    /// Set the action of the entry under the cursor
    SetAction(crate::git::rebase::RebaseAction),
    /// Move the entry under the cursor up one line
    MoveEntryUp,
    /// Move the entry under the cursor down one line
    MoveEntryDown,
    /// Undo the last edit
    Undo,
    /// Close the editor without rebasing
    Abort,
    /// Enter vim-style command mode (`:`)
    CommandStart,
    /// Type a character into the command line
    CommandChar(char),
    /// Delete the last character from the command line (exits on empty)
    CommandBackspace,
    /// Leave command mode without running anything
    CommandCancel,
    /// Confirmed an unrecognised command — show an error and leave command mode
    CommandInvalid,
}

/// Messages for merge commands
#[derive(PartialEq, Eq, Debug)]
pub enum MergeCommand {
    /// Merge the given branch into the current branch
    Branch(String),
    /// Merge the given branch into the current branch, editing the merge message
    /// (`git merge --edit --no-ff <branch>`)
    EditMessage(String),
    /// Merge the given branch without committing
    /// (`git merge --no-commit --no-ff <branch>`)
    NoCommit(String),
    /// Merge the given branch into the current branch, deleting it afterwards
    /// (`git merge --no-edit <branch>` followed by `git branch -D <branch>`)
    Absorb(String),
    /// Preview the result of merging the given branch, without touching the
    /// working tree (`git merge-tree --write-tree HEAD <branch>` diffed
    /// against HEAD)
    Preview(String),
    /// Squash-merge the given branch into the working tree without committing
    /// (`git merge --squash <branch>`)
    Squash(String),
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
    /// Revert the given commit hashes, creating a revert commit.
    /// Honors the revert popup arguments (--edit / --no-edit).
    Commits {
        hashes: Vec<String>,
        mainline: Option<String>,
    },
    /// Run a fully-built `git revert` command that opens the user's editor
    /// for the commit message. Requires the TUI to be suspended.
    WithEditor { args: Vec<String> },
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
    /// Stash only the unstaged worktree changes, keeping the index intact
    /// (no porcelain equivalent — built with plumbing, see git::worktree_stash)
    Worktree,
    /// Stash both index and working tree, keeping the index intact (git stash push --keep-index)
    KeepingIndex,
}

impl StashType {
    /// Human-readable title used in the input popup
    pub fn title(self) -> &'static str {
        let t = i18n::t();
        match self {
            StashType::Both => t.input_stash_message,
            StashType::Index => t.input_stash_index_message,
            StashType::Worktree => t.input_stash_worktree_message,
            StashType::KeepingIndex => t.input_stash_keeping_index_message,
        }
    }

    /// Extra git flag for this stash type, if any
    pub fn flag(self) -> Option<&'static str> {
        match self {
            StashType::Both => None,
            StashType::Index => Some("--staged"),
            StashType::Worktree => None,
            StashType::KeepingIndex => Some("--keep-index"),
        }
    }

    /// Title shown in the PTY output panel
    pub fn pty_title(self) -> &'static str {
        match self {
            StashType::Both => "Stash",
            StashType::Index => "Stash index",
            StashType::Worktree => "Stash worktree",
            StashType::KeepingIndex => "Stash keeping index",
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
    /// Create a snapshot stash of index + working tree without resetting them
    Snapshot,
    /// Create a snapshot stash of only the index, without resetting it
    SnapshotIndex,
    /// Create a snapshot stash of only the unstaged working tree changes, without resetting them
    SnapshotWorktree,
    /// Commit the index and working tree states to the wip refs, without resetting them
    ToWipRef,
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
    /// Show select popup (or confirm) to pick the base commit for an
    /// interactive rebase
    RebaseInteractive,
    /// Show log pick to choose the start commit for a subset rebase
    /// (the new base was already picked in a select popup)
    RebaseSubset { newbase: String },
    /// Show select popup (or confirm) to pick a commit to modify
    /// (stops an interactive rebase at that commit for editing)
    ModifyCommit,
    /// Show select popup (or confirm) to pick a commit to reword
    /// (an interactive rebase marks it `reword` and opens the editor)
    RewordCommit,

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
    /// Apply a text editing operation to the select popup filter
    Edit(EditOp),
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
    /// Apply a text editing operation to the text field
    Edit(EditOp),
    /// Confirm the input
    Confirm,
}

#[derive(PartialEq, Eq, Debug)]
pub enum CredentialsMessage {
    /// Apply a text editing operation to the credential input
    Edit(EditOp),
    /// Confirm credential input (submit the credential)
    CredentialConfirm,
}
