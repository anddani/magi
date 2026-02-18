use git2::Repository;

use crate::{
    errors::MagiResult,
    git::{CommitRef, CommitRefType},
    model::LogEntry,
    msg::LogType,
};

use super::commit_utils::sort_refs;

const MAX_LOG_ENTRIES: usize = 256;
const SEPARATOR: char = '\x0c'; // Form feed character

/// Fetches git log entries with graph
pub fn get_log_entries(repository: &Repository, log_type: LogType) -> MagiResult<Vec<LogEntry>> {
    let workdir = repository
        .workdir()
        .ok_or_else(|| git2::Error::from_str("No working directory"))?;

    // Get list of remote names to distinguish remote branches from local branches with slashes
    let remotes: Vec<String> = repository
        .remotes()
        .map(|r| r.iter().filter_map(|s| s.map(String::from)).collect())
        .unwrap_or_default();

    let head_detached = repository.head_detached()?;

    // Build the git log command similar to Magit
    // Format: hash<sep>refs<sep>author<sep>date<sep>message
    let format = format!(
        "%h{}%D{}%aN{}%ar{}%s",
        SEPARATOR, SEPARATOR, SEPARATOR, SEPARATOR
    );

    let mut args = vec![
        "log".to_string(),
        format!("--format={}", format),
        "--graph".to_string(),
        "--decorate=short".to_string(),
        format!("-n{}", MAX_LOG_ENTRIES),
    ];

    match log_type {
        LogType::Current => args.push("HEAD".to_string()),
        LogType::AllReferences => args.push("--all".to_string()),
        LogType::LocalBranches => {
            if head_detached {
                args.push("HEAD".to_string());
            }
            args.push("--branches".to_string());
        }
        LogType::AllBranches => {
            if head_detached {
                args.push("HEAD".to_string());
            }
            args.push("--branches".to_string());
            args.push("--remotes".to_string());
        }
    }

    args.push("--".to_string());

    let args_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();

    let output = super::git_cmd(workdir, &args_refs).output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(git2::Error::from_str(&format!("git log failed: {}", stderr)).into());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let entries = parse_log_output(&stdout, &remotes);

    Ok(entries)
}

/// Parse the output of git log --graph into LogEntry structs
fn parse_log_output(output: &str, remotes: &[String]) -> Vec<LogEntry> {
    let mut entries = Vec::new();

    for line in output.lines() {
        let entry = parse_log_line(line, remotes);
        entries.push(entry);
    }

    entries
}

fn none_if_empty(s: &str) -> Option<String> {
    if s.is_empty() {
        None
    } else {
        Some(s.to_string())
    }
}

/// Parse a single line from git log --graph output
fn parse_log_line(line: &str, remotes: &[String]) -> LogEntry {
    // The line format is: <graph><hash><sep><refs><sep><author><sep><date><sep><message>
    // The graph part is everything before the first non-graph character that looks like a hash

    // Find where the graph ends and the commit info begins
    // Graph characters are: | / \ * - _ < > . and space
    let graph_end = find_graph_end(line);

    let graph = line[..graph_end].to_string();
    let rest = &line[graph_end..];

    // If no content after graph, it's a graph-only line
    if rest.is_empty() {
        return LogEntry::graph_only(graph);
    }

    // Split by separator
    let parts: Vec<&str> = rest.split(SEPARATOR).collect();

    if parts.len() >= 5 {
        let hash = none_if_empty(parts[0]);
        let refs = parse_refs(parts[1], remotes);
        let author = none_if_empty(parts[2]);
        let time = none_if_empty(parts[3]).and_then(|t| t.strip_suffix(" ago").map(String::from));
        let message = none_if_empty(parts[4]);
        LogEntry::new(graph, hash, refs, author, time, message)
    } else if !parts[0].is_empty() {
        // Has some content but not in expected format
        // Treat as a commit with just a hash/message
        LogEntry::new(
            graph,
            Some(parts[0].to_string()),
            Vec::new(),
            None,
            None,
            parts.get(1).map(|s| s.to_string()),
        )
    } else {
        LogEntry::graph_only(graph)
    }
}

/// Check if a ref name is a remote branch by checking if it starts with a known remote name
fn is_remote_branch(name: &str, remotes: &[String]) -> bool {
    remotes
        .iter()
        .any(|remote| name.starts_with(&format!("{}/", remote)))
}

/// Parse a refs string (e.g., "HEAD -> main, origin/main, tag: v1.0") into CommitRefs
fn parse_refs(refs_str: &str, remotes: &[String]) -> Vec<CommitRef> {
    if refs_str.is_empty() {
        return Vec::new();
    }

    let mut refs = Vec::new();
    let mut current_branch: Option<String> = None;

    // refs can contain multiple references separated by ", "
    // e.g., "HEAD -> main, origin/main, tag: v1.0"
    for part in refs_str
        .split(", ")
        .map(|part| part.trim())
        .filter(|part| !part.is_empty())
    {
        match part {
            "HEAD" => {
                refs.push(CommitRef {
                    name: "@".to_string(),
                    ref_type: CommitRefType::Head,
                });
            }
            p if p.starts_with("HEAD -> ") => {
                // HEAD pointing to a branch - this is the current branch
                let branch_name = part.strip_prefix("HEAD -> ").unwrap_or(part);
                current_branch = Some(branch_name.to_string());
                refs.push(CommitRef {
                    name: branch_name.to_string(),
                    ref_type: CommitRefType::LocalBranch,
                });
            }
            p if p.starts_with("tag: ") => {
                let tag_name = part.strip_prefix("tag: ").unwrap_or(part);
                refs.push(CommitRef {
                    name: tag_name.to_string(),
                    ref_type: CommitRefType::Tag,
                });
            }
            p if is_remote_branch(p, remotes) => {
                // Remote branch (starts with a known remote name)
                refs.push(CommitRef {
                    name: part.to_string(),
                    ref_type: CommitRefType::RemoteBranch,
                });
            }
            _ => {
                // Local branch (including branches with slashes like feature/xyz)
                refs.push(CommitRef {
                    name: part.to_string(),
                    ref_type: CommitRefType::LocalBranch,
                });
            }
        }
    }

    sort_refs(refs, current_branch.as_deref())
}

/// Find where the graph prefix ends in a log line
/// Returns the byte index where the graph ends
fn find_graph_end(line: &str) -> usize {
    let graph_chars = ['|', '/', '\\', '*', '-', '_', '<', '>', '.', ' '];

    let mut end = 0;
    for (i, c) in line.char_indices() {
        if graph_chars.contains(&c) {
            end = i + c.len_utf8();
        } else {
            // Found a non-graph character, stop here
            break;
        }
    }

    end
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_graph_end_simple() {
        assert_eq!(find_graph_end("* abc123"), 2);
        assert_eq!(find_graph_end("| * def456"), 4);
        assert_eq!(find_graph_end("|/  ghi789"), 4);
    }

    #[test]
    fn test_find_graph_end_no_graph() {
        assert_eq!(find_graph_end("abc123"), 0);
    }

    #[test]
    fn test_find_graph_end_only_graph() {
        assert_eq!(find_graph_end("| |"), 3);
        assert_eq!(find_graph_end("* "), 2);
    }

    #[test]
    fn test_parse_log_line_commit() {
        let remotes = vec!["origin".to_string()];
        let line = format!(
            "* abc1234{}main{}John Doe{}2 hours ago{}Fix bug",
            SEPARATOR, SEPARATOR, SEPARATOR, SEPARATOR
        );
        let entry = parse_log_line(&line, &remotes);

        assert_eq!(entry.graph, "* ");
        assert_eq!(entry.hash, Some("abc1234".to_string()));
        assert_eq!(entry.refs.len(), 1);
        assert_eq!(entry.refs[0].name, "main");
        assert_eq!(entry.refs[0].ref_type, CommitRefType::LocalBranch);
        assert_eq!(entry.author, Some("John Doe".to_string()));
        assert_eq!(entry.time, Some("2 hours".to_string()));
        assert_eq!(entry.message, Some("Fix bug".to_string()));
    }

    #[test]
    fn test_parse_log_line_graph_only() {
        let remotes = vec!["origin".to_string()];
        let entry = parse_log_line("| |", &remotes);

        assert_eq!(entry.graph, "| |");
        assert!(entry.hash.is_none());
        assert!(entry.message.is_none());
    }

    #[test]
    fn test_parse_log_line_no_refs() {
        let remotes = vec!["origin".to_string()];
        let line = format!(
            "* def5678{}{}Jane Smith{}1 day ago{}Initial commit",
            SEPARATOR, SEPARATOR, SEPARATOR, SEPARATOR
        );
        let entry = parse_log_line(&line, &remotes);

        assert_eq!(entry.hash, Some("def5678".to_string()));
        assert!(entry.refs.is_empty());
        assert_eq!(entry.author, Some("Jane Smith".to_string()));
    }

    #[test]
    fn test_parse_refs_head_with_branch() {
        let remotes = vec!["origin".to_string()];
        // "HEAD -> main" means main is the current branch, which comes first
        // No "@" is added for non-detached HEAD
        let refs = parse_refs("HEAD -> main, origin/main", &remotes);
        assert_eq!(refs.len(), 2);
        assert_eq!(refs[0].name, "main");
        assert_eq!(refs[0].ref_type, CommitRefType::LocalBranch);
        assert_eq!(refs[1].name, "origin/main");
        assert_eq!(refs[1].ref_type, CommitRefType::RemoteBranch);
    }

    #[test]
    fn test_parse_refs_with_tag() {
        let remotes = vec!["origin".to_string()];
        let refs = parse_refs("tag: v1.0.0", &remotes);
        assert_eq!(refs.len(), 1);
        assert_eq!(refs[0].name, "v1.0.0");
        assert_eq!(refs[0].ref_type, CommitRefType::Tag);
    }

    #[test]
    fn test_parse_refs_empty() {
        let remotes = vec!["origin".to_string()];
        let refs = parse_refs("", &remotes);
        assert!(refs.is_empty());
    }

    #[test]
    fn test_parse_refs_local_branch_with_slash() {
        let remotes = vec!["origin".to_string()];
        // A local branch like "feature/test" should NOT be treated as remote
        let refs = parse_refs("feature/test", &remotes);
        assert_eq!(refs.len(), 1);
        assert_eq!(refs[0].name, "feature/test");
        assert_eq!(refs[0].ref_type, CommitRefType::LocalBranch);
    }

    #[test]
    fn test_parse_refs_distinguishes_local_and_remote_with_slash() {
        let remotes = vec!["origin".to_string(), "upstream".to_string()];
        // origin/main is remote, test/branch is local
        // Local branches come first (sorted), then remote branches (sorted)
        let refs = parse_refs("origin/main, test/branch, upstream/feature", &remotes);
        assert_eq!(refs.len(), 3);
        assert_eq!(refs[0].name, "test/branch");
        assert_eq!(refs[0].ref_type, CommitRefType::LocalBranch);
        assert_eq!(refs[1].name, "origin/main");
        assert_eq!(refs[1].ref_type, CommitRefType::RemoteBranch);
        assert_eq!(refs[2].name, "upstream/feature");
        assert_eq!(refs[2].ref_type, CommitRefType::RemoteBranch);
    }
}
