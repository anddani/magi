use crate::git::CommitRef;

/// A single entry in the git log view
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogEntry {
    /// The graph prefix (lines, branches, asterisks)
    pub graph: String,
    /// The commit hash (abbreviated)
    pub hash: Option<String>,
    /// References pointing to this commit (branches, tags, HEAD)
    pub refs: Vec<CommitRef>,
    /// The author name
    pub author: Option<String>,
    /// The relative time (e.g., "2 days")
    pub time: Option<String>,
    /// The commit message subject
    pub message: Option<String>,
    /// The signature status character (`%G?`: G, B, U, X, Y, R, E or N) when
    /// the log was requested with --show-signature
    pub signature: Option<char>,
}

impl LogEntry {
    /// Create a new LogEntry from parsed components
    pub fn new(
        graph: String,
        hash: Option<String>,
        refs: Vec<CommitRef>,
        author: Option<String>,
        time: Option<String>,
        message: Option<String>,
    ) -> Self {
        Self {
            graph,
            hash,
            refs,
            author,
            time,
            message,
            signature: None,
        }
    }

    /// Create a graph-only entry (for continuation lines)
    pub fn graph_only(graph: String) -> Self {
        Self {
            graph,
            hash: None,
            refs: Vec::new(),
            author: None,
            time: None,
            message: None,
            signature: None,
        }
    }

    /// Returns true if this entry has commit information (not just graph lines)
    pub fn is_commit(&self) -> bool {
        self.hash.is_some()
    }
}
