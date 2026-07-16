# Test Audit: Snapshot Coverage & Code Coverage

Audit conducted 2026-07-15 on commit `de94078`. Coverage numbers are from a real
`cargo llvm-cov` run of the full suite (471+ tests).

Reproduce coverage locally on NixOS:

```sh
nix shell nixpkgs#cargo-llvm-cov nixpkgs#llvm --command sh -c \
  'export LLVM_COV=$(command -v llvm-cov) LLVM_PROFDATA=$(command -v llvm-profdata); cargo llvm-cov --workspace'
```

## TL;DR

- **Overall line coverage: 78.0%** (17,559 lines, 3,856 missed). Functions: 81.9%.
- **Snapshot tests: 28 exist**, all in `tests/render_snapshots.rs`. Popups are well
  covered; the big gaps are the **preview view, sequencer entry states, search/visual
  mode rendering, and empty states**.
- **`msg/update/` is the weakest directory (56.4%)** — mostly because everything that
  executes through the PTY path (push/pull/fetch/revert/apply/amend) has 0% coverage.
- Realistic ceiling without heavy mocking infrastructure: **~93–95%**. True 100% would
  require trait-injected command runners, PTY fakes, and terminal mocks — not worth it.

---

## Part 1: Snapshot test audit

### What exists (28 tests)

Infrastructure (`tests/render_snapshots.rs:30-52`):

- `create_snapshot_model(test_repo)` — model with workdir pinned to `/repo/magi/` for determinism
- `render_to_string(model, w, h)` — renders via ratatui `TestBackend` to plain text
- `render_to_styled_string(...)` — styled buffer variant, for style-sensitive tests
- `assert_frame_snapshot!` — insta snapshot with commit hashes filtered to `[hash]`

Covered surfaces:

| Surface | Tests |
|---|---|
| Status view | 2 (all sections; merge conflict with expanded diff) |
| Log view | 1 (graph + refs + time) |
| Rebase todo view | 1 (mixed pick/fixup/reword/drop) |
| Commit popup | 2 (default, with author) |
| Push popup | 2 (normal, argument mode styled) |
| Fetch / Pull / Branch / Log / Stash / Reset / Tag popups | 1 each |
| Rebase popup | 3 (normal, with push remote, in progress) |
| Revert / Merge / Apply popups | 1 each |
| Select popup | 1 (filtered list) |
| Input popup | 2 (cursor at end, mid-text) |
| Error / Confirm / Credential / Help popups | 1 each |

### Gaps — ordered by priority

These are the "early UI work" surfaces with no snapshot at all. Line coverage confirms
several render as fully untested (0% files noted).

1. **Preview view** (`ViewMode::Preview`) — no snapshot at all.
   `src/view/preview_line.rs` is **0% covered** (all 6 line types: Header,
   DiffFileHeader, HunkHeader, Addition, Deletion, Context). One commit-preview and
   one stash-preview snapshot would cover the whole file.
2. **Status view header refs** — `src/view/merge_ref.rs` and `src/view/push_ref.rs`
   are **0% covered**; `src/view/latest_tag.rs` is **0% covered**. The existing status
   snapshots use a repo without push remote / merge / tags. One richer status-view
   fixture (branch with upstream + pushRemote + tag) covers all three.
3. **Unpulled commits section** — `src/view/unpulled_section_header.rs` is **0%
   covered**. Needs a test repo with a remote-tracking branch ahead of local.
4. **Sequencer sections in status view**:
   - Reverting section (`view/reverting_entry.rs` — covered by unit tests but no
     frame snapshot of the "Reverting" section with stop/pending entries)
   - Cherry-picking section (same for `view/cherry_picking_entry.rs`)
   - Rebasing section mid-rebase (only the todo *view* has a snapshot, not the
     status-view "Rebasing" section)
5. **Search highlighting** — no snapshot of a frame with an active query
   (match highlight spans, bottom-border `/ query` display, n/N navigation state).
6. **Visual mode** — no snapshot of a multi-line visual selection in status or log view.
7. **Select popup states** — only the filtered-with-matches state is snapshotted.
   Missing: no-matches state, unfiltered list, cursor mid-list/scrolled.
8. **Credential popup unmasked** — only the masked (password) variant is snapshotted.
9. **Empty states** — empty repo (no commits), clean working tree, no stashes.
10. **Argument mode for other popups** — only push popup has an arg-mode snapshot;
    fetch/pull/rebase arg highlighting is untested styling.
11. **Status bar modes** — PREVIEW / visual / search labels not pinned by any
    styled snapshot.
12. **Detached HEAD** — log and status views with detached HEAD (`@` ref rendering).

All of these follow the same recipe: build a `TestRepo` in the right state, mutate the
model (popup/view mode/search query), `render_to_string`, `assert_frame_snapshot!`.
Rough effort: **~15–20 new snapshot tests to close every gap above**; the render-side
0% files alone need ~5.

---

## Part 2: Code coverage breakdown

### By directory (lines)

| Directory | Files | Lines | Missed | Coverage |
|---|---|---|---|---|
| `view/render/` | 19 | 976 | 61 | **93.8%** |
| `keys/` (top level) | 4 | 152 | 12 | 92.1% |
| `view/` | 21 | 1,721 | 138 | 92.0% |
| `config/` | 2 | 263 | 26 | 90.1% |
| `git/` | 30 | 5,177 | 673 | 87.0% |
| `model/` | 8 | 1,199 | 169 | 85.9% |
| root (`magi.rs`, `main.rs`, `msg.rs`, ...) | 9 | 2,462 | 411 | 83.3% |
| `msg/` (top level) | 2 | 156 | 43 | 72.4% |
| `git/info/` | 4 | 111 | 37 | 66.7% |
| `keys/command_popup/` | 14 | 524 | 184 | 64.9% |
| **`msg/update/`** | **83** | **4,818** | **2,102** | **56.4%** |
| **TOTAL** | 196 | 17,559 | 3,856 | **78.0%** |

### Zero-coverage files, categorized

**A. PTY / network execution paths (the big block — needs infrastructure to test):**

| File | Lines | Why untested |
|---|---|---|
| `msg/update/push.rs` | 138 | `execute_pty_command` + network |
| `msg/update/revert.rs` | 114 | execution goes through PTY (popup/show handlers ARE tested) |
| `msg/update/fetch.rs` | 99 | PTY + network |
| `msg/update/pull.rs` | 91 | PTY + network |
| `msg/update/apply.rs` | 49 | PTY execution (apply popup itself is 96.8%) |
| `msg/update/pty_helper.rs` | 30 | PTY plumbing |
| `msg/update/amend.rs` | 26 | PTY (editor) |
| `model/pty_state.rs` | 22 | only used by PTY flow |
| `msg/update/credentials_input.rs` | 14 | credential round-trip needs live PTY |
| `keys/credentials_popup.rs` | 7 | same |

Note the pattern: `show_*_popup` handlers and views are tested, but the *execution*
handlers that shell out through the PTY are not. `tests/update_revert.rs` /
`update_apply.rs` exercise the popup and section logic only.

**B. Plain untested handlers — easily testable today with existing helpers:**

| File | Lines | Note |
|---|---|---|
| `msg/update/worktree_checkout.rs` | 33 | TestRepo can create worktrees |
| `msg/update/open_pr.rs` | 25 | split URL logic (already tested in git/open_pr.rs) from browser launch |
| `msg/update/revise_commit.rs` | 21 | |
| `msg/update/checkout_new_branch.rs` | 20 | |
| `msg/update/show_fetch_popup.rs` / `show_pull_popup.rs` / `show_push_popup.rs` | 19 each | trivial popup-state tests |
| `msg/update/show_commit_author_select.rs` | 17 | |
| `msg/update/rename_branch.rs` | 13 | |
| `msg/update/show_revert_mainline_input.rs` | 11 | |
| `msg/update/select_edit.rs` | 8 | |
| `msg/update/select_move_up.rs` / `select_move_down.rs` | 6 each | select popup cursor movement |
| `msg/update/show_log_popup.rs` | 5 | |
| `msg/update/pending_g.rs` | 4 | |

**C. View files at 0% — closed automatically by the Part 1 snapshot gaps:**
`view/latest_tag.rs` (10), `view/preview_line.rs` (15), `view/unpulled_section_header.rs` (17),
`view/merge_ref.rs` (6), `view/push_ref.rs` (6).

**D. Entry points — low value:**
`main.rs` (60 lines, arg parsing — testable if factored into a function),
`magi.rs` at 4.8% (168 lines — terminal init/restore + event loop; not meaningfully testable).

### Worst partially-covered files (biggest missed-line counts)

| File | Coverage | Missed | What's missing |
|---|---|---|---|
| `msg/update/show_select_popup.rs` | 55.9% | 179 | many `OptionsSource`/`OnSelect` arms never exercised |
| `msg/update/selection.rs` | 73.2% | 182 | visual-selection edge branches |
| `msg/update/select_confirm.rs` | 65.8% | 150 | `route_result()` arms for untested flows |
| `magi.rs` | 4.8% | 160 | event loop (skip) |
| `git/stage.rs` | 62.6% | 113 | hunk-edge staging branches |
| `msg/update/rebase.rs` | 20.2% | 83 | continue/skip/abort go through PTY |
| `git/pty_command.rs` | 55.4% | 82 | error paths, credential I/O threads |
| `msg/update/cherry_spinout.rs` / `cherry_spinoff.rs` | ~55% | 57 each | error/edge branches |
| `git/commit.rs` | 65.0% | 63 | fixup/alter paths (PTY editor) |
| `msg/update/stash.rs` | 22.2% | 49 | stash push/pop execution |
| `msg/update/search.rs` | 80.6% | 48 | wrap-around / empty-query branches |
| `msg/update/show_prune_tags_confirm.rs` | 17.3% | 43 | |
| `msg/update/donate.rs` | 61.8% | 39 | |
| `keys/command_popup/branch.rs` | 38.7% | 38 | key arms without tests |
| `git/info/get_latest_tag.rs` | 35.5% | 20 | tag-at-HEAD path never hit |
| `msg/update/toggle_argument.rs` | 46.8% | 25 | |
| `keys/command_popup/stash.rs` | 18.8% | 26 | |
| `keys/command_popup/commit.rs` | 25.0% | 24 | |

Honorable mention: several `view/render/*_popup.rs` files show 50% *function* coverage
with ~97% line coverage — the uncovered "function" is usually a closure or a
never-rendered variant; not worth chasing.

---

## Part 3: Feasibility of 100% coverage

Rough budget of the 3,856 missed lines:

| Tier | Missed lines (approx) | Effort |
|---|---|---|
| **Tier 1 — snapshot gaps (Part 1)**: 0% view files + partially covered view code | ~200 | Trivial: ~15–20 snapshot tests using existing helpers |
| **Tier 2 — untested update handlers (category B) + keys/command_popup arms + show_select_popup/select_confirm arms** | ~900 | Easy-to-moderate: same style as existing `tests/update_*.rs`; mostly mechanical |
| **Tier 3 — git/ edge branches** (stage.rs hunk edges, cherry spinoff/spinout, search branches, prune tags, stash) | ~700 | Moderate: needs crafted repo states, all doable with TestRepo |
| **Tier 4 — PTY/network execution** (category A + rebase/merge/commit PTY arms) | ~1,000 | Hard: needs a `GitCommandRunner`/PTY trait abstraction to fake command execution; OR local-remote integration tests (TestRepo with a file:// remote can cover fetch/pull/push happy paths without mocks) |
| **Tier 5 — not worth it**: `magi.rs` event loop, `main.rs`, terminal init/restore, PTY system error paths, git2 defensive `.ok()` fallbacks (missing HEAD, non-UTF8) | ~1,000 | Would require mocking crossterm/ratatui/git2 or corrupting repos; skip |

Projected coverage by tier (cumulative):

- Tier 1 done: **~79%**
- Tier 2 done: **~84%**
- Tier 3 done: **~88%**
- Tier 4 done: **~94%**
- 100% is **not realistically reachable** — Tier 5 is defensive/OS-level code.
  A sensible target is **90% overall with `msg/update/` above 80%**, plus an
  `#[cfg(coverage)]`-style exclusion or `--ignore-filename-regex` for
  `magi.rs`/`main.rs`/`pty_command.rs` if you want the headline number to be honest.

### Notes on Tier 4 (PTY/network) approaches

Two viable strategies, can be combined:

1. **file:// remote integration tests** — TestRepo already builds real repos; add a
   bare "remote" repo and wire it as `origin`. `git push/fetch/pull` over the file
   protocol needs no credentials, so `execute_pty_command` completes without
   prompting. Covers push.rs, fetch.rs, pull.rs happy paths with zero mocking.
2. **Trait-based command runner** — introduce a small trait (real impl wraps
   `execute_pty_command`) injected via the model, with a recording fake for tests.
   Unlocks error paths and credential-prompt flows deterministically. The credential
   *detection* logic is already well unit-tested (`git/credential.rs`, 11 tests);
   what's missing is the wiring around it.

### Suggested order of attack

1. **Part 1 snapshot gaps** — highest value per test; kills all 0% view files and
   pins early UI work against regressions (~15–20 tests).
2. **Category B handlers** — mechanical, ~15 small tests, +~5% coverage.
3. **`show_select_popup` / `select_confirm` / `keys/command_popup` arms** — table-driven
   tests over the enum arms would close these efficiently.
4. **file:// remote tests for push/fetch/pull** — big block of 0% lines, no mocks needed.
5. Decide whether Tier 4 error paths justify the command-runner abstraction, and
   whether to exclude Tier 5 files from the coverage denominator.
