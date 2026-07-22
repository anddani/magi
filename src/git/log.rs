use git2::Repository;

use crate::{
    errors::MagiResult,
    git::{CommitRef, CommitRefType},
    model::LogEntry,
    msg::LogType,
};

use super::commit_utils::{build_push_remote_map, enrich_refs_with_push_remote, sort_refs};

const MAX_LOG_ENTRIES: usize = 256;
const SEPARATOR: char = '\x0c'; // Form feed character

/// Fetches git log entries, optionally with graph and colored graph lines
pub fn get_log_entries(
    repository: &Repository,
    log_type: &LogType,
    graph: bool,
    color: bool,
) -> MagiResult<Vec<LogEntry>> {
    let workdir = repository
        .workdir()
        .ok_or_else(|| git2::Error::from_str("No working directory"))?;

    // Get list of remote names to distinguish remote branches from local branches with slashes
    let remotes: Vec<String> = repository
        .remotes()
        .map(|r| {
            r.iter()
                .filter_map(|s| s.ok().flatten().map(String::from))
                .collect()
        })
        .unwrap_or_default();

    let head_detached = repository.head_detached()?;

    let reflog = matches!(
        log_type,
        LogType::Reflog | LogType::ReflogOther(_) | LogType::Stashes
    );

    // With no stashes the refs/stash ref doesn't exist and git log would
    // fail; show an empty list instead, like Magit's empty stashes buffer
    if matches!(log_type, LogType::Stashes) && repository.find_reference("refs/stash").is_err() {
        return Ok(Vec::new());
    }

    // Build the git log command similar to Magit
    // Format: hash<sep>refs<sep>author<sep>date<sep>message
    // Reflogs use the reflog subject (%gs, e.g. "commit: fix bug") as message
    let message_format = if reflog { "%gs" } else { "%s" };
    let format = format!(
        "%h{}%D{}%aN{}%ar{}{}",
        SEPARATOR, SEPARATOR, SEPARATOR, SEPARATOR, message_format
    );

    let mut args = vec![
        "log".to_string(),
        format!("--format={}", format),
        "--decorate=short".to_string(),
        format!("-n{}", MAX_LOG_ENTRIES),
    ];

    // git rejects --graph together with --walk-reflogs
    if graph && !reflog {
        args.push("--graph".to_string());
    }

    if color {
        // Without a <when> value, --color defaults to `always`, so the graph
        // is colored even though the output is piped. Only the graph gets
        // ANSI codes since the --format fields don't use %C placeholders.
        args.push("--color".to_string());
    }

    match log_type {
        LogType::Current => args.push("HEAD".to_string()),
        LogType::Other(revision) => args.push(revision.clone()),
        LogType::Related => args.extend(get_related_revs(repository)),
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
        LogType::Reflog => {
            args.push("--walk-reflogs".to_string());
            args.push(reflog_ref(repository));
        }
        LogType::ReflogOther(reference) => {
            args.push("--walk-reflogs".to_string());
            args.push(reference.clone());
        }
        LogType::Stashes => {
            args.push("--walk-reflogs".to_string());
            args.push("refs/stash".to_string());
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
    let mut entries = parse_log_output(&stdout, &remotes);

    // Enrich refs with push remote info (split-colored labels)
    let push_remote_map = build_push_remote_map(repository);
    for entry in &mut entries {
        if !entry.refs.is_empty() {
            let refs = std::mem::take(&mut entry.refs);
            entry.refs = enrich_refs_with_push_remote(refs, &push_remote_map);
        }
    }

    Ok(entries)
}

/// Resolves the ref whose reflog to show, mirroring `magit-reflog-current`:
/// the current branch, or HEAD when detached
fn reflog_ref(repository: &Repository) -> String {
    if repository.head_detached().unwrap_or(true) {
        return "HEAD".to_string();
    }
    repository
        .head()
        .ok()
        .and_then(|head| head.shorthand().ok().map(String::from))
        .unwrap_or_else(|| "HEAD".to_string())
}

/// Resolves the revisions for a related log, mirroring `magit-log-related`:
/// the current branch, its push target, its upstream and — when the upstream
/// is a local branch — that branch's own upstream. When HEAD is detached, the
/// previously checked out branch and HEAD are shown instead.
fn get_related_revs(repository: &Repository) -> Vec<String> {
    let detached = repository.head_detached().unwrap_or(false);

    let current = if detached {
        previous_branch(repository)
    } else {
        repository
            .head()
            .ok()
            .and_then(|head| head.shorthand().ok().map(String::from))
    };

    let mut revs: Vec<String> = Vec::new();
    let mut push_rev = |rev: Option<String>| {
        if let Some(rev) = rev
            && !revs.contains(&rev)
        {
            revs.push(rev);
        }
    };

    match current {
        Some(branch) => {
            push_rev(Some(branch.clone()));
            if detached {
                push_rev(Some("HEAD".to_string()));
            }
            push_rev(push_branch(repository, &branch));
            if let Some((upstream, upstream_is_local)) = upstream_branch(repository, &branch) {
                // When the upstream is a local branch, also show its own upstream
                let upup = if upstream_is_local {
                    upstream_branch(repository, &upstream).map(|(name, _)| name)
                } else {
                    None
                };
                push_rev(Some(upstream));
                push_rev(upup);
            }
        }
        None => push_rev(Some("HEAD".to_string())),
    }

    revs
}

/// Gets the push target of `branch` (e.g. "origin/main") if that remote
/// branch exists
fn push_branch(repository: &Repository, branch: &str) -> Option<String> {
    let remote = super::config::get_push_remote(repository, branch)?;
    let candidate = format!("{}/{}", remote, branch);
    repository
        .find_branch(&candidate, git2::BranchType::Remote)
        .ok()?;
    Some(candidate)
}

/// Gets the upstream of `branch` as a short rev, and whether it is a local
/// branch. Returns None if no upstream is configured or it doesn't exist
fn upstream_branch(repository: &Repository, branch: &str) -> Option<(String, bool)> {
    let refname = repository
        .branch_upstream_name(&format!("refs/heads/{}", branch))
        .ok()?;
    let refname = refname.as_str().ok()?;
    // Only include upstreams that actually exist
    repository.find_reference(refname).ok()?;
    if let Some(local) = refname.strip_prefix("refs/heads/") {
        Some((local.to_string(), true))
    } else {
        refname
            .strip_prefix("refs/remotes/")
            .map(|remote| (remote.to_string(), false))
    }
}

/// Gets the previously checked out branch (`@{-1}`), if it is a local branch
fn previous_branch(repository: &Repository) -> Option<String> {
    let (_, reference) = repository.revparse_ext("@{-1}").ok()?;
    let reference = reference?;
    if reference.is_branch() {
        reference.shorthand().ok().map(String::from)
    } else {
        None
    }
}

/// Parse the output of git log into LogEntry structs
/// Works with both --graph and non-graph output
pub fn parse_log_output(output: &str, remotes: &[String]) -> Vec<LogEntry> {
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
                    push_remote: None,
                });
            }
            p if p.starts_with("HEAD -> ") => {
                // HEAD pointing to a branch - this is the current branch
                let branch_name = part.strip_prefix("HEAD -> ").unwrap_or(part);
                current_branch = Some(branch_name.to_string());
                refs.push(CommitRef {
                    name: branch_name.to_string(),
                    ref_type: CommitRefType::LocalBranch,
                    push_remote: None,
                });
            }
            p if p.starts_with("tag: ") => {
                let tag_name = part.strip_prefix("tag: ").unwrap_or(part);
                refs.push(CommitRef {
                    name: tag_name.to_string(),
                    ref_type: CommitRefType::Tag,
                    push_remote: None,
                });
            }
            p if is_remote_branch(p, remotes) => {
                // Skip remote HEAD symbolic refs (e.g., origin/HEAD)
                if p.ends_with("/HEAD") {
                    continue;
                }
                // Remote branch (starts with a known remote name)
                refs.push(CommitRef {
                    name: part.to_string(),
                    ref_type: CommitRefType::RemoteBranch,
                    push_remote: None,
                });
            }
            _ => {
                // Local branch (including branches with slashes like feature/xyz)
                refs.push(CommitRef {
                    name: part.to_string(),
                    ref_type: CommitRefType::LocalBranch,
                    push_remote: None,
                });
            }
        }
    }

    sort_refs(refs, current_branch.as_deref())
}

/// Find where the graph prefix ends in a log line
/// Returns the byte index where the graph ends
/// ANSI escape sequences (from --color) count as part of the graph
fn find_graph_end(line: &str) -> usize {
    let graph_chars = ['|', '/', '\\', '*', '-', '_', '<', '>', '.', ' '];

    let mut end = 0;
    let mut i = 0;
    while i < line.len() {
        if line.as_bytes()[i] == 0x1b {
            // ANSI escape sequence (e.g. "\x1b[31m") coloring the graph
            match line[i..].find('m') {
                Some(m) => {
                    i += m + 1;
                    end = i;
                }
                None => break,
            }
        } else {
            let c = line[i..].chars().next().unwrap();
            if !graph_chars.contains(&c) {
                // Found a non-graph character, stop here
                break;
            }
            i += c.len_utf8();
            end = i;
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
    fn test_find_graph_end_with_ansi_codes() {
        // Colored graph from git log --graph --color
        let line = "\x1b[31m|\x1b[m \x1b[32m*\x1b[m abc123";
        assert_eq!(find_graph_end(line), line.len() - "abc123".len());
        // Graph-only line where the color codes cover the whole line
        let line = "\x1b[31m|\x1b[m \x1b[32m|\x1b[m";
        assert_eq!(find_graph_end(line), line.len());
    }

    #[test]
    fn test_find_graph_end_only_graph() {
        assert_eq!(find_graph_end("| |"), 3);
        assert_eq!(find_graph_end("* "), 2);
    }

    #[test]
    fn test_get_log_entries_other_revision() {
        use crate::git::test_repo::TestRepo;

        let test_repo = TestRepo::new();
        // Create a branch at the initial commit, then advance main past it
        let head = test_repo.repo.head().unwrap().peel_to_commit().unwrap();
        test_repo.repo.branch("feature", &head, false).unwrap();
        test_repo
            .write_file_content("file.txt", "content")
            .stage_files(&["file.txt"])
            .commit("Second commit");

        let entries = get_log_entries(
            &test_repo.repo,
            &LogType::Other("feature".to_string()),
            true,
            false,
        )
        .unwrap();
        let messages: Vec<String> = entries.iter().filter_map(|e| e.message.clone()).collect();

        assert!(messages.contains(&"Initial commit".to_string()));
        assert!(!messages.contains(&"Second commit".to_string()));
    }

    #[test]
    fn test_get_log_entries_reflog() {
        use crate::git::test_repo::TestRepo;

        let test_repo = TestRepo::new();
        test_repo
            .write_file_content("file.txt", "content")
            .stage_files(&["file.txt"])
            .commit("Second commit");

        let entries = get_log_entries(&test_repo.repo, &LogType::Reflog, true, false).unwrap();
        let messages: Vec<String> = entries.iter().filter_map(|e| e.message.clone()).collect();

        // Reflog entries use the reflog subject ("<command>: <rest>")
        assert!(messages.iter().any(|m| m.contains("Second commit")));
        assert!(messages.iter().any(|m| m.starts_with("commit")));
        // No graph is drawn even though graph was requested
        assert!(entries.iter().all(|e| e.graph.is_empty()));
    }

    #[test]
    fn test_get_log_entries_reflog_other() {
        use crate::git::test_repo::TestRepo;

        let test_repo = TestRepo::new();
        test_repo
            .write_file_content("file.txt", "content")
            .stage_files(&["file.txt"])
            .commit("Second commit")
            .create_branch("feature");

        let entries = get_log_entries(
            &test_repo.repo,
            &LogType::ReflogOther("feature".to_string()),
            true,
            false,
        )
        .unwrap();
        let messages: Vec<String> = entries.iter().filter_map(|e| e.message.clone()).collect();

        // The branch's reflog only has its creation entry
        assert!(messages.iter().any(|m| m.starts_with("branch")));
        // No graph is drawn even though graph was requested
        assert!(entries.iter().all(|e| e.graph.is_empty()));
    }

    #[test]
    fn test_get_log_entries_stashes() {
        use crate::git::test_repo::TestRepo;

        let test_repo = TestRepo::new();
        test_repo
            .write_file_content("file.txt", "first")
            .create_stash("First stash");
        test_repo
            .write_file_content("file.txt", "second")
            .create_stash("Second stash");

        let entries = get_log_entries(&test_repo.repo, &LogType::Stashes, true, false).unwrap();
        let messages: Vec<String> = entries.iter().filter_map(|e| e.message.clone()).collect();

        // Both stashes are listed, most recent first
        assert_eq!(messages.len(), 2);
        assert!(messages[0].contains("Second stash"));
        assert!(messages[1].contains("First stash"));
        // No graph is drawn even though graph was requested
        assert!(entries.iter().all(|e| e.graph.is_empty()));
    }

    #[test]
    fn test_get_log_entries_stashes_empty_without_stashes() {
        use crate::git::test_repo::TestRepo;

        let test_repo = TestRepo::new();

        let entries = get_log_entries(&test_repo.repo, &LogType::Stashes, true, false).unwrap();
        assert!(entries.is_empty());
    }

    #[test]
    fn test_reflog_ref_current_branch() {
        use crate::git::test_repo::TestRepo;

        let test_repo = TestRepo::new();
        assert_eq!(reflog_ref(&test_repo.repo), "main");
    }

    #[test]
    fn test_reflog_ref_detached_head() {
        use crate::git::test_repo::TestRepo;

        let test_repo = TestRepo::new();
        test_repo.detach_head();
        assert_eq!(reflog_ref(&test_repo.repo), "HEAD");
    }

    #[test]
    fn test_get_related_revs_current_branch_only() {
        use crate::git::test_repo::TestRepo;

        let test_repo = TestRepo::new();
        let revs = get_related_revs(&test_repo.repo);

        assert_eq!(revs, vec!["main".to_string()]);
    }

    #[test]
    fn test_get_related_revs_includes_remote_upstream_and_push_target() {
        use crate::git::test_repo::TestRepo;

        let test_repo = TestRepo::new();
        let head = test_repo
            .repo
            .head()
            .unwrap()
            .peel_to_commit()
            .unwrap()
            .id();

        // Simulate a remote with origin/main as both upstream and push target
        test_repo
            .repo
            .remote("origin", "https://example.com/repo.git")
            .unwrap();
        test_repo
            .repo
            .reference("refs/remotes/origin/main", head, false, "remote branch")
            .unwrap();
        let mut config = test_repo.repo.config().unwrap();
        config.set_str("branch.main.remote", "origin").unwrap();
        config
            .set_str("branch.main.merge", "refs/heads/main")
            .unwrap();
        config.set_str("remote.pushDefault", "origin").unwrap();

        let revs = get_related_revs(&test_repo.repo);

        // Upstream and push target are the same rev, so it only appears once
        assert_eq!(revs, vec!["main".to_string(), "origin/main".to_string()]);
    }

    #[test]
    fn test_get_related_revs_local_upstream_includes_its_own_upstream() {
        use crate::git::test_repo::TestRepo;

        let test_repo = TestRepo::new();
        let head = test_repo
            .repo
            .head()
            .unwrap()
            .peel_to_commit()
            .unwrap()
            .id();

        // main tracks origin/main
        test_repo
            .repo
            .remote("origin", "https://example.com/repo.git")
            .unwrap();
        test_repo
            .repo
            .reference("refs/remotes/origin/main", head, false, "remote branch")
            .unwrap();
        let mut config = test_repo.repo.config().unwrap();
        config.set_str("branch.main.remote", "origin").unwrap();
        config
            .set_str("branch.main.merge", "refs/heads/main")
            .unwrap();

        // feature tracks the local branch main
        test_repo.create_branch("feature");
        test_repo.repo.set_head("refs/heads/feature").unwrap();
        config.set_str("branch.feature.remote", ".").unwrap();
        config
            .set_str("branch.feature.merge", "refs/heads/main")
            .unwrap();

        let revs = get_related_revs(&test_repo.repo);

        assert_eq!(
            revs,
            vec![
                "feature".to_string(),
                "main".to_string(),
                "origin/main".to_string()
            ]
        );
    }

    #[test]
    fn test_get_related_revs_detached_head_uses_previous_branch() {
        use crate::git::test_repo::TestRepo;

        let test_repo = TestRepo::new();
        test_repo.create_branch("feature");
        test_repo.repo.set_head("refs/heads/feature").unwrap();
        test_repo.detach_head();

        let revs = get_related_revs(&test_repo.repo);

        assert_eq!(revs, vec!["feature".to_string(), "HEAD".to_string()]);
    }

    #[test]
    fn test_get_log_entries_related() {
        use crate::git::test_repo::TestRepo;

        let test_repo = TestRepo::new();
        // Create a branch at the initial commit, make it main's local upstream,
        // then advance main past it
        let head = test_repo.repo.head().unwrap().peel_to_commit().unwrap();
        test_repo.repo.branch("base", &head, false).unwrap();
        let mut config = test_repo.repo.config().unwrap();
        config.set_str("branch.main.remote", ".").unwrap();
        config
            .set_str("branch.main.merge", "refs/heads/base")
            .unwrap();
        test_repo
            .write_file_content("file.txt", "content")
            .stage_files(&["file.txt"])
            .commit("Second commit");

        let entries = get_log_entries(&test_repo.repo, &LogType::Related, true, false).unwrap();
        let messages: Vec<String> = entries.iter().filter_map(|e| e.message.clone()).collect();

        assert!(messages.contains(&"Initial commit".to_string()));
        assert!(messages.contains(&"Second commit".to_string()));
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
    fn test_parse_log_line_colored_graph() {
        let remotes = vec!["origin".to_string()];
        let line = format!(
            "\x1b[31m|\x1b[m * abc1234{}main{}John Doe{}2 hours ago{}Fix bug",
            SEPARATOR, SEPARATOR, SEPARATOR, SEPARATOR
        );
        let entry = parse_log_line(&line, &remotes);

        assert_eq!(entry.graph, "\x1b[31m|\x1b[m * ");
        assert_eq!(entry.hash, Some("abc1234".to_string()));
        assert_eq!(entry.message, Some("Fix bug".to_string()));
    }

    #[test]
    fn test_get_log_entries_color() {
        use crate::git::test_repo::TestRepo;

        let test_repo = TestRepo::new();
        test_repo
            .write_file_content("file.txt", "content")
            .stage_files(&["file.txt"])
            .commit("Second commit");

        let entries = get_log_entries(&test_repo.repo, &LogType::Current, true, true).unwrap();

        // The graph is colored with ANSI codes, but the commit info is not
        let entry = entries.first().unwrap();
        assert!(entry.graph.contains('*'));
        assert_eq!(entry.message, Some("Second commit".to_string()));
        assert!(!entry.message.as_ref().unwrap().contains('\x1b'));
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
    fn test_parse_refs_filters_remote_head() {
        let remotes = vec!["origin".to_string()];
        let refs = parse_refs("origin/HEAD, origin/main", &remotes);
        assert_eq!(refs.len(), 1);
        assert_eq!(refs[0].name, "origin/main");
        assert_eq!(refs[0].ref_type, CommitRefType::RemoteBranch);
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
