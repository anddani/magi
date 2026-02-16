use std::process::Command;

use git2::Repository;

use crate::{errors::MagiResult, model::LogEntry};

const MAX_LOG_ENTRIES: usize = 256;
const SEPARATOR: char = '\x0c'; // Form feed character

/// Fetches git log entries with graph for the current branch
pub fn get_log_entries(repository: &Repository) -> MagiResult<Vec<LogEntry>> {
    let workdir = repository
        .workdir()
        .ok_or_else(|| git2::Error::from_str("No working directory"))?;

    // Build the git log command similar to Magit
    // Format: hash<sep>refs<sep>author<sep>date<sep>message
    let format = format!(
        "%h{}%D{}%aN{}%ar{}%s",
        SEPARATOR, SEPARATOR, SEPARATOR, SEPARATOR
    );

    let output = Command::new("git")
        .arg("-C")
        .arg(workdir)
        .arg("log")
        .arg(format!("--format={}", format))
        .arg("--graph")
        .arg("--decorate=short")
        .arg(format!("-n{}", MAX_LOG_ENTRIES))
        .arg("HEAD")
        .arg("--")
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(git2::Error::from_str(&format!("git log failed: {}", stderr)).into());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let entries = parse_log_output(&stdout);

    Ok(entries)
}

/// Parse the output of git log --graph into LogEntry structs
fn parse_log_output(output: &str) -> Vec<LogEntry> {
    let mut entries = Vec::new();

    for line in output.lines() {
        let entry = parse_log_line(line);
        entries.push(entry);
    }

    entries
}

/// Parse a single line from git log --graph output
fn parse_log_line(line: &str) -> LogEntry {
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
        let hash = if parts[0].is_empty() {
            None
        } else {
            Some(parts[0].to_string())
        };
        let refs = if parts[1].is_empty() {
            None
        } else {
            Some(parts[1].to_string())
        };
        let author = if parts[2].is_empty() {
            None
        } else {
            Some(parts[2].to_string())
        };
        let time = if parts[3].is_empty() {
            None
        } else {
            Some(parts[3].to_string())
        };
        let message = if parts[4].is_empty() {
            None
        } else {
            Some(parts[4].to_string())
        };

        LogEntry::new(graph, hash, refs, author, time, message)
    } else if !parts[0].is_empty() {
        // Has some content but not in expected format
        // Treat as a commit with just a hash/message
        LogEntry::new(
            graph,
            Some(parts[0].to_string()),
            None,
            None,
            None,
            parts.get(1).map(|s| s.to_string()),
        )
    } else {
        LogEntry::graph_only(graph)
    }
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
        let line = format!(
            "* abc1234{}main{}John Doe{}2 hours ago{}Fix bug",
            SEPARATOR, SEPARATOR, SEPARATOR, SEPARATOR
        );
        let entry = parse_log_line(&line);

        assert_eq!(entry.graph, "* ");
        assert_eq!(entry.hash, Some("abc1234".to_string()));
        assert_eq!(entry.refs, Some("main".to_string()));
        assert_eq!(entry.author, Some("John Doe".to_string()));
        assert_eq!(entry.time, Some("2 hours ago".to_string()));
        assert_eq!(entry.message, Some("Fix bug".to_string()));
    }

    #[test]
    fn test_parse_log_line_graph_only() {
        let entry = parse_log_line("| |");

        assert_eq!(entry.graph, "| |");
        assert!(entry.hash.is_none());
        assert!(entry.message.is_none());
    }

    #[test]
    fn test_parse_log_line_no_refs() {
        let line = format!(
            "* def5678{}{}Jane Smith{}1 day ago{}Initial commit",
            SEPARATOR, SEPARATOR, SEPARATOR, SEPARATOR
        );
        let entry = parse_log_line(&line);

        assert_eq!(entry.hash, Some("def5678".to_string()));
        assert!(entry.refs.is_none());
        assert_eq!(entry.author, Some("Jane Smith".to_string()));
    }
}
