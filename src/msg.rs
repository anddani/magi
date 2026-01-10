pub mod update;
mod util;

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
    /// Toggle section expand/collapse
    ToggleSection,

    /// User initiated a commit
    UserCommit,
    /// Open commit in user's default EDITOR
    OpenCommitEditor,

    /// Dismiss the current dialog
    DismissDialog,
    /// Stage all modified files (does not include untracked files)
    StageAllModified,
    /// Unstage all staged files
    UnstageAll,
}
