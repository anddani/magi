use std::collections::HashSet;
use std::time::Instant;

use crate::config::Theme;
use crate::git::{GitInfo, GitRef, TagInfo};
use crate::msg::Message;

/// The whole state of the application, including the Git repository Handle
pub struct Model {
    /// Running state of the application
    pub running_state: RunningState,
    /// The model passed to the view function to render the main UI.
    pub ui_model: UiModel,
    /// git2 Git repository Handle
    pub git_info: GitInfo,
    /// Magi color scheme constants
    pub theme: Theme,
    /// Modal dialog that requires user action to dismiss
    pub dialog: Option<DialogContent>,
    /// Toast notification that auto-dismisses after a timeout
    pub toast: Option<Toast>,
}

#[derive(Debug, Clone)]
pub struct Toast {
    pub message: String,
    pub style: ToastStyle,
    pub expires_at: Instant,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)]
pub enum ToastStyle {
    Success,
    Info,
    Warning,
}

#[derive(Default, Clone)]
pub struct UiModel {
    pub lines: Vec<Line>,
    pub cursor_position: usize,
    pub scroll_offset: usize,
    pub viewport_height: usize,
    pub collapsed_sections: HashSet<SectionType>,
}

#[derive(Debug, Clone)]
pub enum LineContent {
    EmptyLine,
    HeadRef(GitRef),
    PushRef(GitRef),
    Tag(TagInfo),
    SectionHeader { title: String, count: Option<usize> },
    UntrackedFile(String),
    UnstagedFile(FileChange),
    StagedFile(FileChange),
    DiffHunk(DiffHunk),
    DiffLine(DiffLine),
}

/// Represents a file change (modified, deleted, renamed, etc.)
#[derive(Debug, Clone)]
pub struct FileChange {
    pub path: String,
    pub status: FileStatus,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FileStatus {
    Modified,
    Deleted,
    New,
    Renamed,
    Copied,
    TypeChange,
}

/// Represents a diff hunk header (e.g., @@ -7,6 +7,7 @@)
#[derive(Debug, Clone)]
pub struct DiffHunk {
    pub header: String,
}

/// Represents a single line in a diff
#[derive(Debug, Clone)]
pub struct DiffLine {
    pub content: String,
    pub line_type: DiffLineType,
}

/// Addition and Deletion lines should be prefixed
/// with + and - and highlighted.
#[derive(Debug, Clone, PartialEq)]
pub enum DiffLineType {
    Context,
    Addition,
    Deletion,
}

#[derive(Debug, Clone)]
pub struct Line {
    pub content: LineContent,
    /// A line has an association with a section in order to
    /// collapse and expand lines inside a section.
    pub section: Option<SectionType>,
}

impl Line {
    /// Returns true if this line should be hidden because its section is collapsed
    pub fn is_hidden(&self, collapsed_sections: &HashSet<SectionType>) -> bool {
        if let Some(ref section) = self.section {
            // Check if a parent section is collapsed
            if section.is_hidden_by(collapsed_sections) {
                return true;
            }
            // Also check if this section itself is collapsed AND this is not a header line.
            // Headers (SectionHeader, UnstagedFile, HeadRef) should remain visible when collapsed.
            if collapsed_sections.contains(section)
                && !matches!(
                    self.content,
                    LineContent::SectionHeader { .. }
                        | LineContent::UnstagedFile(_)
                        | LineContent::StagedFile(_)
                        | LineContent::HeadRef(_)
                )
            {
                return true;
            }
        }
        false
    }

    /// Returns the section type to toggle when this line is a header.
    /// Returns None if this line is not a collapsible header.
    pub fn collapsible_section(&self) -> Option<SectionType> {
        match (&self.content, &self.section) {
            (LineContent::SectionHeader { .. }, Some(section)) => Some(section.clone()),
            (LineContent::HeadRef(_), _) => Some(SectionType::Info),
            (LineContent::UnstagedFile(file_change), _) => Some(SectionType::UnstagedFile {
                path: file_change.path.clone(),
            }),
            (LineContent::StagedFile(file_change), _) => Some(SectionType::StagedFile {
                path: file_change.path.clone(),
            }),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SectionType {
    Info,
    UntrackedFiles,
    /// The main "Unstaged changes" section header
    UnstagedChanges,
    /// A file within unstaged changes (selecting highlights all hunks)
    UnstagedFile {
        path: String,
    },
    /// A specific hunk within a file
    UnstagedHunk {
        path: String,
        hunk_index: usize,
    },
    /// The main "Staged changes" section header
    StagedChanges,
    /// A file within staged changes (selecting highlights all hunks)
    StagedFile {
        path: String,
    },
    /// A specific hunk within a staged file
    StagedHunk {
        path: String,
        hunk_index: usize,
    },
}

impl SectionType {
    /// Returns the parent section that can be collapsed to hide this section.
    /// For example, UnstagedHunk's parent is UnstagedFile, UnstagedFile's parent is UnstagedChanges.
    pub fn parent_section(&self) -> Option<SectionType> {
        match self {
            SectionType::Info => None,
            SectionType::UntrackedFiles => None,
            SectionType::UnstagedChanges => None,
            SectionType::UnstagedFile { .. } => Some(SectionType::UnstagedChanges),
            SectionType::UnstagedHunk { path, .. } => {
                Some(SectionType::UnstagedFile { path: path.clone() })
            }
            SectionType::StagedChanges => None,
            SectionType::StagedFile { .. } => Some(SectionType::StagedChanges),
            SectionType::StagedHunk { path, .. } => {
                Some(SectionType::StagedFile { path: path.clone() })
            }
        }
    }

    /// Checks if this section should be hidden because a parent is collapsed.
    pub fn is_hidden_by(&self, collapsed: &HashSet<SectionType>) -> bool {
        // Check if any parent section is collapsed
        let mut current = self.parent_section();
        while let Some(parent) = current {
            if collapsed.contains(&parent) {
                return true;
            }
            current = parent.parent_section();
        }
        false
    }

    /// When the application starts, we don't want to expand all sections.
    pub fn default_collapsed(&self) -> bool {
        matches!(
            self,
            SectionType::StagedFile { .. } | SectionType::UnstagedFile { .. }
        )
    }
}

#[derive(Default, PartialEq, Eq, Debug)]
pub enum RunningState {
    #[default]
    Running,
    /// Signal to main loop to launch the an external command
    /// so that it can pause the Ratatui rendering and then
    /// resume it when the application returns to [`Running`].
    LaunchExternalCommand(Message),
    Done,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DialogContent {
    Error { message: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_section_type_parent_section() {
        // Top-level sections have no parent
        assert_eq!(SectionType::Info.parent_section(), None);
        assert_eq!(SectionType::UntrackedFiles.parent_section(), None);
        assert_eq!(SectionType::UnstagedChanges.parent_section(), None);

        // UnstagedFile's parent is UnstagedChanges
        let file_section = SectionType::UnstagedFile {
            path: "foo.rs".to_string(),
        };
        assert_eq!(
            file_section.parent_section(),
            Some(SectionType::UnstagedChanges)
        );

        // UnstagedHunk's parent is UnstagedFile
        let hunk_section = SectionType::UnstagedHunk {
            path: "foo.rs".to_string(),
            hunk_index: 0,
        };
        assert_eq!(
            hunk_section.parent_section(),
            Some(SectionType::UnstagedFile {
                path: "foo.rs".to_string()
            })
        );
    }

    #[test]
    fn test_is_hidden_by_collapsed_parent() {
        let mut collapsed = HashSet::new();

        let file_section = SectionType::UnstagedFile {
            path: "foo.rs".to_string(),
        };
        let hunk_section = SectionType::UnstagedHunk {
            path: "foo.rs".to_string(),
            hunk_index: 0,
        };

        // Nothing collapsed, nothing hidden
        assert!(!file_section.is_hidden_by(&collapsed));
        assert!(!hunk_section.is_hidden_by(&collapsed));

        // Collapse UnstagedChanges, files should be hidden
        collapsed.insert(SectionType::UnstagedChanges);
        assert!(file_section.is_hidden_by(&collapsed));
        assert!(hunk_section.is_hidden_by(&collapsed)); // Hunk is also hidden (grandparent collapsed)

        // Reset and collapse only the file
        collapsed.clear();
        collapsed.insert(SectionType::UnstagedFile {
            path: "foo.rs".to_string(),
        });
        assert!(!file_section.is_hidden_by(&collapsed)); // File itself is not hidden
        assert!(hunk_section.is_hidden_by(&collapsed)); // Hunk is hidden

        // Different file's hunk should not be hidden
        let other_hunk = SectionType::UnstagedHunk {
            path: "bar.rs".to_string(),
            hunk_index: 0,
        };
        assert!(!other_hunk.is_hidden_by(&collapsed));
    }

    #[test]
    fn test_line_is_hidden() {
        let mut collapsed = HashSet::new();
        collapsed.insert(SectionType::UnstagedChanges);

        // Line with collapsed parent section should be hidden
        let line = Line {
            content: LineContent::UnstagedFile(FileChange {
                path: "foo.rs".to_string(),
                status: FileStatus::Modified,
            }),
            section: Some(SectionType::UnstagedFile {
                path: "foo.rs".to_string(),
            }),
        };
        assert!(line.is_hidden(&collapsed));

        // Line with no section should not be hidden
        let empty_line = Line {
            content: LineContent::EmptyLine,
            section: None,
        };
        assert!(!empty_line.is_hidden(&collapsed));

        // Header line should not be hidden (only its children)
        let header_line = Line {
            content: LineContent::SectionHeader {
                title: "Unstaged changes".to_string(),
                count: Some(1),
            },
            section: Some(SectionType::UnstagedChanges),
        };
        assert!(!header_line.is_hidden(&collapsed));

        // UnstagedFile line should not be hidden when its own section is collapsed
        // (it acts as a header for its hunks)
        collapsed.clear();
        collapsed.insert(SectionType::UnstagedFile {
            path: "foo.rs".to_string(),
        });
        let file_line = Line {
            content: LineContent::UnstagedFile(FileChange {
                path: "foo.rs".to_string(),
                status: FileStatus::Modified,
            }),
            section: Some(SectionType::UnstagedFile {
                path: "foo.rs".to_string(),
            }),
        };
        assert!(!file_line.is_hidden(&collapsed));
    }

    #[test]
    fn test_collapsible_section() {
        // SectionHeader returns its section
        let header_line = Line {
            content: LineContent::SectionHeader {
                title: "Untracked files".to_string(),
                count: Some(2),
            },
            section: Some(SectionType::UntrackedFiles),
        };
        assert_eq!(
            header_line.collapsible_section(),
            Some(SectionType::UntrackedFiles)
        );

        // HeadRef returns Info section
        let head_ref_line = Line {
            content: LineContent::HeadRef(crate::git::GitRef::new(
                "main".to_string(),
                "abc1234".to_string(),
                "Initial commit".to_string(),
                crate::git::ReferenceType::LocalBranch,
            )),
            section: Some(SectionType::Info),
        };
        assert_eq!(head_ref_line.collapsible_section(), Some(SectionType::Info));

        // UnstagedFile returns UnstagedFile section
        let file_line = Line {
            content: LineContent::UnstagedFile(FileChange {
                path: "foo.rs".to_string(),
                status: FileStatus::Modified,
            }),
            section: Some(SectionType::UnstagedFile {
                path: "foo.rs".to_string(),
            }),
        };
        assert_eq!(
            file_line.collapsible_section(),
            Some(SectionType::UnstagedFile {
                path: "foo.rs".to_string()
            })
        );

        // Other content types return None
        let diff_line = Line {
            content: LineContent::DiffLine(DiffLine {
                content: "+ added".to_string(),
                line_type: DiffLineType::Addition,
            }),
            section: Some(SectionType::UnstagedHunk {
                path: "foo.rs".to_string(),
                hunk_index: 0,
            }),
        };
        assert_eq!(diff_line.collapsible_section(), None);

        // Empty line returns None
        let empty_line = Line {
            content: LineContent::EmptyLine,
            section: None,
        };
        assert_eq!(empty_line.collapsible_section(), None);
    }
}
