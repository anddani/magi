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
    /// Toggle section expand/collapse
    ToggleSection,

    /// Open commit in user's default EDITOR
    Commit,

    /// Dismiss the current dialog
    DismissDialog,
    /// Stage all modified files (does not include untracked files)
    StageAllModified,
    /// Unstage all staged files
    UnstageAll,

    /// Enter visual selection mode (sets anchor at current cursor position)
    EnterVisualMode,
    /// Exit visual selection mode (clears the anchor)
    ExitVisualMode,
}
