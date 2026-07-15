# Performance Audit

Research into magi performance, conducted 2026-07-15. Benchmarks were run against a
local nixpkgs clone (957k commits, 2.5 GB `.git`, 6 refs). Line numbers refer to the
code as of commit `de94078`.

## TL;DR

1. The nixpkgs log hang is `git log --graph` forcing a full topological sort of all
   957k commits before printing anything. A git commit-graph file makes it instant.
2. Diffs and previews are loaded fully into memory with no size limits.
3. Every mutation triggers a full synchronous re-scan of all ten status sections.
4. The render loop does per-frame work that scales with total content (and calls
   git2 per visible commit line).
5. A few subprocess-per-item loops (discard, unmerged) that should be batched.

---

## 1. The nixpkgs hang: `--graph` topological sort

### Benchmarks

Exact command magi runs (`git log --format=%h%x0c%D%x0c%aN%x0c%ar%x0c%s --decorate=short -n256`):

| Invocation | Time | Peak RSS |
|---|---|---|
| `-n256 HEAD` (no graph) | 0.00s | 51 MB |
| `-n256 --graph HEAD` | **5.64s** | **1.4 GB** |
| `-n256 --graph HEAD` + commit-graph file | 0.01s | 108 MB |
| `-n256 --graph --color --all` + commit-graph | 0.01s | 108 MB |

### Why

`--graph` implies `--topo-order`. A topological sort requires walking **all** reachable
commits before the first line can be emitted — the `-n256` limit is applied *after*
sorting, so it doesn't help. The 1.4 GB is git holding the entire commit DAG in memory.

The commit-graph file (`.git/objects/info/commit-graph`) caches the DAG with generation
numbers, letting git stream topo-ordered output incrementally. Building it is a one-time
cost (`git commit-graph write --reachable` took 8.6s on nixpkgs) and git maintains it
incrementally afterward (gc writes it by default since git 2.24; `fetch.writeCommitGraph=true`
keeps it fresh on fetch).

Note: a commit-graph was written into the local nixpkgs clone during this audit and left
in place. It is a pure cache; delete `.git/objects/info/commit-graph` to undo.

### What magi can do

- [ ] When the log is slow / repo is large and no commit-graph exists, build one in a
      background thread: `git commit-graph write --reachable --changed-paths`.
      This is what makes Magit-scale workflows viable on huge repos.
- [ ] `src/msg/update/show_commit_select.rs:105` hardcodes `graph=true` for the
      commit-picker popups (rebase elsewhere, harvest, ...). These don't need the graph
      at all — pass `false`. Free win.
- [ ] The log popup enables graph by default (`src/msg/update/show_log_popup.rs:11`).
      Consider falling back to no-graph when the repo is large and has no commit-graph.

### Not affected

The status view is safe in huge repos: `src/git/recent_commits.rs` uses a lazy git2
revwalk with `.take(10)` (no topo sort). Its ref-map building (tag map, branch maps,
push-remote map) is O(refs) per refresh — trivial with few refs, worth caching only
if repos with thousands of refs become a target.

---

## 2. Diffs and previews are fully materialized, unbounded

### Status diffs

`collect_file_changes` (`src/git/diff_utils.rs:13-96`) eagerly walks **every hunk and
every line of every changed file** into
`Vec<(FileChange, Vec<(DiffHunk, Vec<DiffLine>)>)>`, one heap `String` per diff line
(plus a second allocation from `trim_end_matches('\n').to_string()`).

- No `DiffOptions::max_size()` is set (`src/git/unstaged_changes.rs:16`,
  `src/git/staged_changes.rs:14`) — a 100 MB changed text file (generated JSON,
  lockfiles) becomes ~100 MB+ of model strings. git2's binary auto-detection only
  saves true binaries.
- Both staged and unstaged diffs are computed and fully materialized on every refresh.
- Rough memory shape: 500 modified files × 50 lines ≈ 4–5 MB of model; 500k diff lines
  ≈ ~110 MB.

Mitigations:

- [ ] Set `DiffOptions::max_size()` to skip huge files and render a
      "File too large" placeholder (Magit does the same).
- [ ] Longer term: lazy-load hunk content per file on expand instead of eagerly for
      all files.

### Previews

`src/git/preview.rs:48-68`: `git show <hash>` (and `git stash show -p`) stdout is read
into a single `String` with no limit — a treewide nixpkgs commit can be tens of MB,
parsed into one `Line` per line.

- [ ] Cap preview output (truncate after N lines with a "... truncated" marker), or
      show `--stat` first and load the full patch on demand.

---

## 3. Full synchronous refresh after every mutation

Every stage/unstage/commit/etc. returns `Message::Refresh` →
`refresh_status` (`src/msg/update/refresh.rs:24`) → `git_info.get_lines()`
(`src/git.rs:83-124`), which recomputes **all ten sections** on the UI thread:

- Two full diffs (staged + unstaged), fully materialized (see §2).
- **Two separate** `Repository::statuses()` working-tree scans:
  `src/git/untracked_files.rs:17` and `src/git/unmerged_changes.rs:20`. These could be
  a single scan serving both consumers.
- Sequencer-state file I/O (rebase/revert/cherry-pick), stash iteration, two revwalks
  (recent + unpulled commits), tag/branch map building.

Staging a single hunk re-diffs the entire working tree. On a big working tree this is
the freeze felt on every action. Fixes in ascending effort:

- [ ] Merge the two `statuses()` scans into one call.
- [ ] Section-selective refresh: only recompute the sections a given action can affect
      (staging a hunk can't change stashes or unpulled commits).
- [ ] Move refresh to a background thread; post the finished `Vec<Line>` back as a
      message. Fits the Elm architecture naturally and unblocks input.

---

## 4. Render-loop costs that scale with content, not viewport

The event loop (`src/magi.rs:135-210`, 250ms poll timeout) redraws on every iteration,
and `view()` rebuilds everything from the model each frame:

- `src/view.rs:96-216` iterates **all** model lines each frame (hidden ones are
  skipped but still visited), allocating fresh `Vec<TextLine>`/`Span`/`String` per
  visible line. No caching, no dirty tracking.
- **`model.git_info.current_branch()` is called inside the per-line render loop**
  (`src/view.rs:155` and `src/view.rs:161`) — a git2 HEAD lookup per visible
  commit/log line, per frame. Hoist above the loop; one-line fix.
- Search highlighting (`apply_search_highlight`, `src/view/util.rs:33-132`) clones
  every span's content to owned `String`s and rebuilds all spans, for every visible
  line, every frame while a query is active.
- `apply_selection_style` (`src/view/util.rs:306-326`) clones spans and allocates a
  padding string per selected line per frame.
- `visible_scroll_offset` (`src/view/util.rs:154-167`) linearly scans from line 0 to
  the scroll offset each frame, with recursive `is_hidden` checks.
- `save_log_return_state` (`src/model.rs:71-83`) clones the entire `UiModel`
  (including all lines) when entering log view.

Invisible in small repos; adds up with thousands of diff lines loaded. Cursor movement
and section toggle themselves are O(1) — the cost is the unconditional full re-render.

- [ ] Hoist `current_branch()` (and similar per-frame git2 lookups) out of the loop.
- [ ] Only build `TextLine`s for the visible viewport slice.
- [ ] Consider caching rendered lines with invalidation on model change.

---

## 5. Subprocess-per-item loops

Each `git` spawn is ~5–50 ms; loops multiply that:

- `src/git/discard.rs:203-213` — one `git rm -f -- <file>` per file.
- `src/git/discard.rs:273-283` — one `git diff -U0 -- <file>` per file.
- `src/git/unmerged_changes.rs:39` — one `git diff -- <path>` per conflicted file.

Discarding 100 files ≈ 0.5–5 s of pure process spawning. Git accepts multiple
pathspecs per invocation — batch them.

- [ ] Batch pathspecs into single git invocations.

---

## 6. Minor findings

- `list_authors` (`src/git/commit.rs:59`) uses `-n9999` and reads full stdout into
  memory. Fine in practice; silently truncates authors on huge repos. Could use
  `git shortlog -sne` or accept the limit.
- `get_all_branches` (`src/git/checkout.rs:33-58`) loads every local + remote branch
  for each select popup. O(branches) per popup open — only matters with thousands of
  refs.
- Ref-map building in `src/git/commit_utils.rs` (tag map, local/remote branch maps,
  push-remote map with a config read per branch) runs on every refresh via
  `recent_commits.rs` and `unpulled_commits.rs`. O(refs); cache if large-ref repos
  become a target.

### Checked and NOT a problem

- `get_latest_tag` (`src/git/info/get_latest_tag.rs`): `graph_ahead_behind` only runs
  when a tag points exactly at HEAD, so it's trivial despite looking expensive. Its
  `tag_foreach` is O(tags). (An earlier automated finding flagged this as costly —
  verified false.)
- `log.rs` output size: 256 entries ≈ ≤256 KB, fine.
- Untracked-file scan respects `.gitignore` (`include_ignored(false)`), so `target/`
  etc. are skipped as long as they're ignored.

---

## Suggested order of attack

1. **Commit-graph handling** + drop `graph=true` from `show_commit_select` — fixes the
   actual hang (§1).
2. **`DiffOptions::max_size` + preview truncation** — bounds worst-case memory (§2).
3. **Hoist `current_branch()` out of the render loop** — one line (§4).
4. **Merge the duplicate `statuses()` calls; batch the discard subprocess loops** (§3, §5).
5. Longer-term: **section-selective refresh** and/or **background refresh thread** (§3),
   viewport-only rendering (§4).
