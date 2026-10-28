use magi::model::InputField;
use std::collections::HashSet;

use magi::{
    git::credential::CredentialType,
    model::{
        Line, LineContent, PreviewLineType, ViewMode,
        arguments::{
            Arguments, LogArgument, MergeArgument, PushArgument, RebaseArgument, RebaseMergesMode,
            TagArgument,
        },
        multi_select_popup::{MultiSelectPopupState, OnMultiSelect},
        popup::{
            ApplyPopupState, CommitPopupState, ConfirmAction, ConfirmPopupState,
            CredentialPopupState, FetchPopupState, InputContext, InputPopupState, MergePopupState,
            PopupContent, PopupContentCommand, PullPopupState, PushPopupState, RebasePopupState,
            RevertPopupState,
        },
        select_popup::{OnSelect, SelectPopupState},
    },
    msg::{CommitSelect, LogType, Message, update::update},
    view::view,
};
use ratatui::{Terminal, backend::TestBackend};

mod utils;
use utils::create_model_from_test_repo;

use magi::git::test_repo::TestRepo;
use magi::model::Model;

/// Create a model whose workdir is pinned to a fixed path so the rendered
/// title bar does not leak the per-run temp directory into snapshots.
fn create_snapshot_model(test_repo: &TestRepo) -> Model {
    let mut model = create_model_from_test_repo(test_repo);
    model.workdir = std::path::PathBuf::from("/repo/magi/");
    model
}

/// Render the full TUI for the given model into a plain-text frame.
fn render_to_string(model: &Model, width: u16, height: u16) -> String {
    let backend = TestBackend::new(width, height);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal.draw(|f| view(model, f)).unwrap();
    terminal.backend().to_string()
}

/// Render the full TUI into the buffer's debug format, which includes the
/// style spans. Used where a state change is only visible through styling
/// (e.g. argument mode highlights).
fn render_to_styled_string(model: &Model, width: u16, height: u16) -> String {
    let backend = TestBackend::new(width, height);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal.draw(|f| view(model, f)).unwrap();
    format!("{:?}", terminal.backend().buffer())
}

/// Snapshot with commit hashes redacted (7-40 hex chars surrounded by word
/// boundaries) and the crate version redacted (help popup) so frames stay
/// deterministic across commits and version bumps.
macro_rules! assert_frame_snapshot {
    ($frame:expr) => {
        insta::with_settings!({filters => vec![
            (r"\b[0-9a-f]{7,40}\b", "[hash]"),
            (r"version \d+\.\d+\.\d+", "version [version]"),
        ]}, {
            insta::assert_snapshot!($frame);
        })
    };
}

// ── Full views ────────────────────────────────────────────────────────────────

#[test]
fn snapshot_status_view_with_all_sections() {
    let test_repo = TestRepo::new();
    test_repo
        .commit_file("committed.txt", "content", "Add committed file")
        .write_file_content("committed.txt", "modified content")
        .commit_file("staged.txt", "staged", "Add staged file")
        .write_file_content("staged.txt", "staged modified")
        .stage_files(&["staged.txt"])
        .create_file("untracked.txt");

    let model = create_snapshot_model(&test_repo);
    assert_frame_snapshot!(render_to_string(&model, 80, 24));
}

#[test]
fn snapshot_status_view_with_unmerged_into() {
    let test_repo = TestRepo::new();

    // Configure an upstream (origin/main) pinned at the initial commit...
    let upstream_oid = test_repo
        .repo
        .head()
        .unwrap()
        .peel_to_commit()
        .unwrap()
        .id();
    test_repo
        .repo
        .remote("origin", "https://example.com/repo.git")
        .unwrap();
    test_repo
        .repo
        .reference(
            "refs/remotes/origin/main",
            upstream_oid,
            false,
            "create remote branch",
        )
        .unwrap();
    test_repo
        .repo
        .find_branch("main", git2::BranchType::Local)
        .unwrap()
        .set_upstream(Some("origin/main"))
        .unwrap();

    // ...then advance the local branch so there is a commit unmerged into the upstream.
    test_repo.commit_file("feature.txt", "feature", "Add feature");

    let model = create_snapshot_model(&test_repo);
    assert_frame_snapshot!(render_to_string(&model, 80, 24));
}

#[test]
fn snapshot_status_view_with_pushremote_sections() {
    let test_repo = TestRepo::new();
    let base_oid = test_repo
        .repo
        .head()
        .unwrap()
        .peel_to_commit()
        .unwrap()
        .id();

    test_repo
        .repo
        .remote("origin", "https://example.com/repo.git")
        .unwrap();
    // A push branch (origin/main) and a *different* upstream branch (origin/upstream),
    // both pinned behind HEAD so each produces its own outgoing section.
    test_repo
        .repo
        .reference("refs/remotes/origin/main", base_oid, false, "push branch")
        .unwrap();
    test_repo
        .repo
        .reference(
            "refs/remotes/origin/upstream",
            base_oid,
            false,
            "upstream branch",
        )
        .unwrap();
    test_repo
        .repo
        .config()
        .unwrap()
        .set_str("branch.main.pushRemote", "origin")
        .unwrap();
    test_repo
        .repo
        .find_branch("main", git2::BranchType::Local)
        .unwrap()
        .set_upstream(Some("origin/upstream"))
        .unwrap();

    // Advance HEAD so both the push branch and the upstream are behind by one commit.
    test_repo.commit_file("feature.txt", "feature", "Add feature");

    let model = create_snapshot_model(&test_repo);
    assert_frame_snapshot!(render_to_string(&model, 80, 24));
}

#[test]
fn snapshot_status_view_with_merge_conflict() {
    let test_repo = TestRepo::new();
    test_repo.create_merge_conflict("conflict.txt");

    let mut model = create_snapshot_model(&test_repo);
    // Expand the unmerged file so the combined diff with conflict markers is visible
    model
        .ui_model
        .collapsed_sections
        .remove(&magi::model::SectionType::UnstagedFile {
            path: "conflict.txt".to_string(),
        });
    assert_frame_snapshot!(render_to_string(&model, 80, 24));
}

#[test]
fn snapshot_help_popup() {
    let test_repo = TestRepo::new();
    let mut model = create_snapshot_model(&test_repo);
    model.popup = Some(PopupContent::Help);
    assert_frame_snapshot!(render_to_string(&model, 80, 24));
}

#[test]
fn snapshot_log_view() {
    let test_repo = TestRepo::new();
    test_repo
        .commit_file("first.txt", "one", "Add first file")
        .commit_file("second.txt", "two", "Add second file");

    let mut model = create_snapshot_model(&test_repo);
    update(&mut model, Message::ShowLog(LogType::Current));

    // Pin the relative commit times so the frame stays deterministic.
    for line in &mut model.ui_model.lines {
        if let LineContent::LogLine(entry) = &mut line.content {
            entry.time = entry.time.as_ref().map(|_| "2 days".to_string());
        }
    }
    assert_frame_snapshot!(render_to_string(&model, 80, 24));
}

#[test]
fn snapshot_log_view_with_header() {
    let test_repo = TestRepo::new();
    test_repo
        .commit_file("first.txt", "one", "Add first file")
        .commit_file("second.txt", "two", "Add second file");

    let mut model = create_snapshot_model(&test_repo);
    model.arguments = Some(Arguments::LogArguments(HashSet::from([
        LogArgument::Graph,
        LogArgument::Decorate,
        LogArgument::ShowHeader,
    ])));
    update(&mut model, Message::ShowLog(LogType::Current));

    // Pin the relative commit times so the frame stays deterministic.
    for line in &mut model.ui_model.lines {
        if let LineContent::LogLine(entry) = &mut line.content {
            entry.time = entry.time.as_ref().map(|_| "2 days".to_string());
        }
    }
    assert_frame_snapshot!(render_to_string(&model, 80, 24));
}

#[test]
fn snapshot_log_pick_view_rebase_subset() {
    let test_repo = TestRepo::new();
    test_repo
        .commit_file("first.txt", "one", "Add first file")
        .commit_file("second.txt", "two", "Add second file");

    let mut model = create_snapshot_model(&test_repo);
    update(
        &mut model,
        Message::ShowCommitSelect(CommitSelect::RebaseSubset {
            newbase: "main".to_string(),
        }),
    );

    // Pin the relative commit times so the frame stays deterministic.
    for line in &mut model.ui_model.lines {
        if let LineContent::LogLine(entry) = &mut line.content {
            entry.time = entry.time.as_ref().map(|_| "2 days".to_string());
        }
    }
    assert_frame_snapshot!(render_to_string(&model, 100, 24));
}

#[test]
fn snapshot_log_pick_view_modify_commit() {
    let test_repo = TestRepo::new();
    test_repo
        .commit_file("first.txt", "one", "Add first file")
        .commit_file("second.txt", "two", "Add second file");

    let mut model = create_snapshot_model(&test_repo);
    update(
        &mut model,
        Message::ShowCommitSelect(CommitSelect::ModifyCommit),
    );

    // Pin the relative commit times so the frame stays deterministic.
    for line in &mut model.ui_model.lines {
        if let LineContent::LogLine(entry) = &mut line.content {
            entry.time = entry.time.as_ref().map(|_| "2 days".to_string());
        }
    }
    assert_frame_snapshot!(render_to_string(&model, 100, 24));
}

#[test]
fn snapshot_log_pick_view_reword_commit() {
    let test_repo = TestRepo::new();
    test_repo
        .commit_file("first.txt", "one", "Add first file")
        .commit_file("second.txt", "two", "Add second file");

    let mut model = create_snapshot_model(&test_repo);
    update(
        &mut model,
        Message::ShowCommitSelect(CommitSelect::RewordCommit),
    );

    // Pin the relative commit times so the frame stays deterministic.
    for line in &mut model.ui_model.lines {
        if let LineContent::LogLine(entry) = &mut line.content {
            entry.time = entry.time.as_ref().map(|_| "2 days".to_string());
        }
    }
    assert_frame_snapshot!(render_to_string(&model, 100, 24));
}

#[test]
fn snapshot_log_pick_view_remove_commit() {
    let test_repo = TestRepo::new();
    test_repo
        .commit_file("first.txt", "one", "Add first file")
        .commit_file("second.txt", "two", "Add second file");

    let mut model = create_snapshot_model(&test_repo);
    update(
        &mut model,
        Message::ShowCommitSelect(CommitSelect::RemoveCommit),
    );

    // Pin the relative commit times so the frame stays deterministic.
    for line in &mut model.ui_model.lines {
        if let LineContent::LogLine(entry) = &mut line.content {
            entry.time = entry.time.as_ref().map(|_| "2 days".to_string());
        }
    }
    assert_frame_snapshot!(render_to_string(&model, 100, 24));
}

#[test]
fn snapshot_log_pick_view_autosquash() {
    let test_repo = TestRepo::new();
    test_repo
        .commit_file("first.txt", "one", "Add first file")
        .commit_file("second.txt", "two", "Add second file");

    // Without an upstream, autosquash falls back to picking the commit to
    // squash into.
    let mut model = create_snapshot_model(&test_repo);
    update(
        &mut model,
        Message::ShowCommitSelect(CommitSelect::Autosquash),
    );

    // Pin the relative commit times so the frame stays deterministic.
    for line in &mut model.ui_model.lines {
        if let LineContent::LogLine(entry) = &mut line.content {
            entry.time = entry.time.as_ref().map(|_| "2 days".to_string());
        }
    }
    assert_frame_snapshot!(render_to_string(&model, 100, 24));
}

// ── Command popups ────────────────────────────────────────────────────────────

/// Create a snapshot model showing the given command popup.
fn create_command_popup_model(test_repo: &TestRepo, command: PopupContentCommand) -> Model {
    let mut model = create_snapshot_model(test_repo);
    model.popup = Some(PopupContent::Command(command));
    model
}

#[test]
fn snapshot_commit_popup() {
    let test_repo = TestRepo::new();
    let model = create_command_popup_model(
        &test_repo,
        PopupContentCommand::Commit(CommitPopupState::default()),
    );
    assert_frame_snapshot!(render_to_string(&model, 80, 24));
}

#[test]
fn snapshot_commit_popup_with_author() {
    let test_repo = TestRepo::new();
    let model = create_command_popup_model(
        &test_repo,
        PopupContentCommand::Commit(CommitPopupState {
            author: Some("André Danielsson <andre@example.com>".to_string()),
            reuse_message: None,
        }),
    );
    assert_frame_snapshot!(render_to_string(&model, 80, 24));
}

#[test]
fn snapshot_commit_popup_with_reuse_message() {
    let test_repo = TestRepo::new();
    let model = create_command_popup_model(
        &test_repo,
        PopupContentCommand::Commit(CommitPopupState {
            author: None,
            reuse_message: Some("ORIG_HEAD".to_string()),
        }),
    );
    assert_frame_snapshot!(render_to_string(&model, 80, 24));
}

#[test]
fn snapshot_push_popup() {
    let test_repo = TestRepo::new();
    let model = create_command_popup_model(
        &test_repo,
        PopupContentCommand::Push(PushPopupState {
            upstream: Some("origin/main".to_string()),
            push_remote: None,
            sole_remote: Some("origin".to_string()),
        }),
    );
    assert_frame_snapshot!(render_to_string(&model, 80, 24));
}

#[test]
fn snapshot_push_popup_arg_mode() {
    let test_repo = TestRepo::new();
    let mut model = create_command_popup_model(
        &test_repo,
        PopupContentCommand::Push(PushPopupState {
            upstream: Some("origin/main".to_string()),
            push_remote: None,
            sole_remote: Some("origin".to_string()),
        }),
    );
    model.arg_mode = true;
    model.arguments = Some(Arguments::PushArguments(HashSet::from([
        PushArgument::ForceWithLease,
    ])));
    // Argument mode only changes styling (key highlights, selected flags), so
    // snapshot the styled buffer instead of the plain-text frame.
    assert_frame_snapshot!(render_to_styled_string(&model, 80, 24));
}

#[test]
fn snapshot_fetch_popup() {
    let test_repo = TestRepo::new();
    let model = create_command_popup_model(
        &test_repo,
        PopupContentCommand::Fetch(FetchPopupState {
            upstream: Some("origin/main".to_string()),
            push_remote: None,
            sole_remote: Some("origin".to_string()),
        }),
    );
    assert_frame_snapshot!(render_to_string(&model, 80, 24));
}

#[test]
fn snapshot_pull_popup() {
    let test_repo = TestRepo::new();
    let model = create_command_popup_model(
        &test_repo,
        PopupContentCommand::Pull(PullPopupState {
            upstream: Some("origin/main".to_string()),
            push_remote: None,
            sole_remote: Some("origin".to_string()),
        }),
    );
    assert_frame_snapshot!(render_to_string(&model, 80, 24));
}

#[test]
fn snapshot_branch_popup() {
    let test_repo = TestRepo::new();
    let model = create_command_popup_model(&test_repo, PopupContentCommand::Branch);
    assert_frame_snapshot!(render_to_string(&model, 80, 24));
}

#[test]
fn snapshot_worktree_popup() {
    let test_repo = TestRepo::new();
    let model = create_command_popup_model(&test_repo, PopupContentCommand::Worktree);
    assert_frame_snapshot!(render_to_string(&model, 80, 24));
}

#[test]
fn snapshot_log_popup() {
    let test_repo = TestRepo::new();
    let model = create_command_popup_model(&test_repo, PopupContentCommand::Log);
    assert_frame_snapshot!(render_to_string(&model, 80, 24));
}

#[test]
fn snapshot_log_popup_arg_mode() {
    let test_repo = TestRepo::new();
    let mut model = create_command_popup_model(&test_repo, PopupContentCommand::Log);
    model.arg_mode = true;
    // Graph and decorate are the defaults set when the popup opens
    model.arguments = Some(Arguments::LogArguments(HashSet::from([
        LogArgument::Graph,
        LogArgument::Decorate,
    ])));
    // Argument mode only changes styling (key highlights, selected flags), so
    // snapshot the styled buffer instead of the plain-text frame.
    assert_frame_snapshot!(render_to_styled_string(&model, 80, 24));
}

#[test]
fn snapshot_log_popup_equals_arg_mode() {
    let test_repo = TestRepo::new();
    let mut model = create_command_popup_model(&test_repo, PopupContentCommand::Log);
    model.equals_arg_mode = true;
    model.arguments = Some(Arguments::LogArguments(HashSet::from([
        LogArgument::Graph,
        LogArgument::Decorate,
        LogArgument::ShowSignature,
    ])));
    // Equals-arg mode only changes styling (key highlights, selected flags),
    // so snapshot the styled buffer instead of the plain-text frame.
    assert_frame_snapshot!(render_to_styled_string(&model, 80, 24));
}

#[test]
fn snapshot_stash_popup() {
    let test_repo = TestRepo::new();
    let model = create_command_popup_model(&test_repo, PopupContentCommand::Stash);
    assert_frame_snapshot!(render_to_string(&model, 80, 24));
}

#[test]
fn snapshot_stash_push_popup() {
    let test_repo = TestRepo::new();
    let model = create_command_popup_model(&test_repo, PopupContentCommand::StashPush);
    assert_frame_snapshot!(render_to_string(&model, 80, 24));
}

#[test]
fn snapshot_multi_select_popup() {
    let test_repo = TestRepo::new();
    let mut state = MultiSelectPopupState::new(
        "Select files to stash (SPACE toggles, RET confirms)".to_string(),
        vec![
            "src/main.rs".to_string(),
            "src/model.rs".to_string(),
            "README.md".to_string(),
        ],
        OnMultiSelect::StashPushFiles {
            flags: vec!["--include-untracked".to_string()],
        },
    );
    // Check the first and third file so the checkboxes render.
    state.checked.insert(0);
    state.checked.insert(2);
    let model = create_command_popup_model(&test_repo, PopupContentCommand::MultiSelect(state));
    assert_frame_snapshot!(render_to_string(&model, 80, 24));
}

#[test]
fn snapshot_reset_popup() {
    let test_repo = TestRepo::new();
    let model = create_command_popup_model(&test_repo, PopupContentCommand::Reset);
    assert_frame_snapshot!(render_to_string(&model, 80, 24));
}

#[test]
fn snapshot_rebase_popup() {
    let test_repo = TestRepo::new();
    let model = create_command_popup_model(
        &test_repo,
        PopupContentCommand::Rebase(RebasePopupState {
            branch: "main".to_string(),
            in_progress: false,
            upstream: None,
            push_remote: None,
            sole_remote: None,
        }),
    );
    assert_frame_snapshot!(render_to_string(&model, 80, 24));
}

#[test]
fn snapshot_rebase_popup_with_push_remote() {
    let test_repo = TestRepo::new();
    let model = create_command_popup_model(
        &test_repo,
        PopupContentCommand::Rebase(RebasePopupState {
            branch: "main".to_string(),
            in_progress: false,
            upstream: None,
            push_remote: Some("origin".to_string()),
            sole_remote: None,
        }),
    );
    assert_frame_snapshot!(render_to_string(&model, 80, 24));
}

#[test]
fn snapshot_rebase_popup_in_progress() {
    let test_repo = TestRepo::new();
    let model = create_command_popup_model(
        &test_repo,
        PopupContentCommand::Rebase(RebasePopupState {
            branch: "main".to_string(),
            in_progress: true,
            upstream: None,
            push_remote: None,
            sole_remote: None,
        }),
    );
    assert_frame_snapshot!(render_to_string(&model, 80, 24));
}

#[test]
fn snapshot_rebase_popup_arg_mode() {
    let test_repo = TestRepo::new();
    let mut model = create_command_popup_model(
        &test_repo,
        PopupContentCommand::Rebase(RebasePopupState {
            branch: "main".to_string(),
            in_progress: false,
            upstream: None,
            push_remote: None,
            sole_remote: None,
        }),
    );
    model.arg_mode = true;
    model.arguments = Some(Arguments::RebaseArguments(HashSet::from([
        RebaseArgument::KeepEmpty,
    ])));
    // Argument mode only changes styling (key highlights, selected flags), so
    // snapshot the styled buffer instead of the plain-text frame.
    assert_frame_snapshot!(render_to_styled_string(&model, 80, 24));
}

#[test]
fn snapshot_rebase_popup_rebase_merges_selected() {
    let test_repo = TestRepo::new();
    let mut model = create_command_popup_model(
        &test_repo,
        PopupContentCommand::Rebase(RebasePopupState {
            branch: "main".to_string(),
            in_progress: false,
            upstream: None,
            push_remote: None,
            sole_remote: None,
        }),
    );
    model.arguments = Some(Arguments::RebaseArguments(HashSet::from([
        RebaseArgument::RebaseMerges(RebaseMergesMode::NoRebaseCousins),
    ])));
    // The selected mode is rendered as the flag value (--rebase-merges=<mode>)
    assert_frame_snapshot!(render_to_string(&model, 80, 24));
}

#[test]
fn snapshot_rebase_popup_update_refs_selected() {
    let test_repo = TestRepo::new();
    let mut model = create_command_popup_model(
        &test_repo,
        PopupContentCommand::Rebase(RebasePopupState {
            branch: "main".to_string(),
            in_progress: false,
            upstream: None,
            push_remote: None,
            sole_remote: None,
        }),
    );
    model.arg_mode = true;
    model.arguments = Some(Arguments::RebaseArguments(HashSet::from([
        RebaseArgument::UpdateRefs,
    ])));
    // Styled buffer shows the selected --update-refs flag highlighted
    assert_frame_snapshot!(render_to_styled_string(&model, 80, 24));
}

#[test]
fn snapshot_rebase_popup_committer_date_is_author_date_selected() {
    let test_repo = TestRepo::new();
    let mut model = create_command_popup_model(
        &test_repo,
        PopupContentCommand::Rebase(RebasePopupState {
            branch: "main".to_string(),
            in_progress: false,
            upstream: None,
            push_remote: None,
            sole_remote: None,
        }),
    );
    model.arg_mode = true;
    model.arguments = Some(Arguments::RebaseArguments(HashSet::from([
        RebaseArgument::CommitterDateIsAuthorDate,
    ])));
    // Styled buffer shows the selected --committer-date-is-author-date flag highlighted
    assert_frame_snapshot!(render_to_styled_string(&model, 80, 24));
}

#[test]
fn snapshot_rebase_popup_ignore_date_selected() {
    let test_repo = TestRepo::new();
    let mut model = create_command_popup_model(
        &test_repo,
        PopupContentCommand::Rebase(RebasePopupState {
            branch: "main".to_string(),
            in_progress: false,
            upstream: None,
            push_remote: None,
            sole_remote: None,
        }),
    );
    model.arg_mode = true;
    model.arguments = Some(Arguments::RebaseArguments(HashSet::from([
        RebaseArgument::IgnoreDate,
    ])));
    // Styled buffer shows the selected --ignore-date flag highlighted
    assert_frame_snapshot!(render_to_styled_string(&model, 80, 24));
}

#[test]
fn snapshot_rebase_popup_autosquash_selected() {
    let test_repo = TestRepo::new();
    let mut model = create_command_popup_model(
        &test_repo,
        PopupContentCommand::Rebase(RebasePopupState {
            branch: "main".to_string(),
            in_progress: false,
            upstream: None,
            push_remote: None,
            sole_remote: None,
        }),
    );
    model.arg_mode = true;
    model.arguments = Some(Arguments::RebaseArguments(HashSet::from([
        RebaseArgument::Autosquash,
    ])));
    // Styled buffer shows the selected --autosquash flag highlighted
    assert_frame_snapshot!(render_to_styled_string(&model, 80, 24));
}

#[test]
fn snapshot_rebase_popup_autostash_selected() {
    let test_repo = TestRepo::new();
    let mut model = create_command_popup_model(
        &test_repo,
        PopupContentCommand::Rebase(RebasePopupState {
            branch: "main".to_string(),
            in_progress: false,
            upstream: None,
            push_remote: None,
            sole_remote: None,
        }),
    );
    model.arg_mode = true;
    model.arguments = Some(Arguments::RebaseArguments(HashSet::from([
        RebaseArgument::Autostash,
    ])));
    // Styled buffer shows the selected --autostash flag highlighted
    assert_frame_snapshot!(render_to_styled_string(&model, 80, 24));
}

#[test]
fn snapshot_rebase_popup_disable_hooks_selected() {
    let test_repo = TestRepo::new();
    let mut model = create_command_popup_model(
        &test_repo,
        PopupContentCommand::Rebase(RebasePopupState {
            branch: "main".to_string(),
            in_progress: false,
            upstream: None,
            push_remote: None,
            sole_remote: None,
        }),
    );
    model.arg_mode = true;
    model.arguments = Some(Arguments::RebaseArguments(HashSet::from([
        RebaseArgument::DisableHooks,
    ])));
    // Styled buffer shows the selected --no-verify flag highlighted
    assert_frame_snapshot!(render_to_styled_string(&model, 80, 24));
}

#[test]
fn snapshot_rebase_todo_view() {
    use magi::git::rebase::RebaseAction;
    use magi::msg::RebaseTodoMessage;

    let test_repo = TestRepo::new();
    test_repo.commit_file("a.txt", "a", "Add feature A");
    let base = test_repo.head_hash();
    test_repo
        .commit_file("b.txt", "b", "Fix typo in A")
        .commit_file("c.txt", "c", "Add feature C")
        .commit_file("d.txt", "d", "WIP experiment");

    // Open the editor with the real workdir (it runs git), then pin the
    // workdir so the title bar stays deterministic.
    let mut model = create_model_from_test_repo(&test_repo);
    update(&mut model, Message::ShowRebaseTodo(base));
    model.workdir = std::path::PathBuf::from("/repo/magi/");

    // Entry 0 stays pick; set actions on the rest (cursor auto-advances)
    model.ui_model.cursor_position = 1;
    for action in [
        RebaseAction::Fixup,
        RebaseAction::Reword,
        RebaseAction::Drop,
    ] {
        update(
            &mut model,
            Message::RebaseTodo(RebaseTodoMessage::SetAction(action)),
        );
    }

    assert_frame_snapshot!(render_to_string(&model, 100, 24));
}

#[test]
fn snapshot_revert_popup() {
    let test_repo = TestRepo::new();
    let model = create_command_popup_model(
        &test_repo,
        PopupContentCommand::Revert(RevertPopupState {
            in_progress: false,
            selected_commits: vec!["1234567890abcdef1234567890abcdef12345678".to_string()],
            mainline: None,
            strategy: None,
            gpg_sign: None,
        }),
    );
    assert_frame_snapshot!(render_to_string(&model, 80, 24));
}

#[test]
fn snapshot_revert_popup_with_strategy() {
    let test_repo = TestRepo::new();
    let model = create_command_popup_model(
        &test_repo,
        PopupContentCommand::Revert(RevertPopupState {
            in_progress: false,
            selected_commits: vec!["1234567890abcdef1234567890abcdef12345678".to_string()],
            mainline: None,
            strategy: Some("recursive".to_string()),
            gpg_sign: None,
        }),
    );
    assert_frame_snapshot!(render_to_string(&model, 80, 24));
}

#[test]
fn snapshot_revert_popup_with_gpg_sign() {
    let test_repo = TestRepo::new();
    let model = create_command_popup_model(
        &test_repo,
        PopupContentCommand::Revert(RevertPopupState {
            in_progress: false,
            selected_commits: vec!["1234567890abcdef1234567890abcdef12345678".to_string()],
            mainline: None,
            strategy: None,
            gpg_sign: Some("ABCD1234".to_string()),
        }),
    );
    assert_frame_snapshot!(render_to_string(&model, 80, 24));
}

#[test]
fn snapshot_revert_strategy_select_popup() {
    let test_repo = TestRepo::new();
    let mut model = create_command_popup_model(
        &test_repo,
        PopupContentCommand::Revert(RevertPopupState {
            in_progress: false,
            selected_commits: vec!["1234567890abcdef1234567890abcdef12345678".to_string()],
            mainline: None,
            strategy: None,
            gpg_sign: None,
        }),
    );
    update(&mut model, Message::ShowRevertStrategySelect);
    assert_frame_snapshot!(render_to_string(&model, 80, 24));
}

#[test]
fn snapshot_merge_popup() {
    let test_repo = TestRepo::new();
    let model = create_command_popup_model(
        &test_repo,
        PopupContentCommand::Merge(MergePopupState { in_progress: false }),
    );
    assert_frame_snapshot!(render_to_string(&model, 80, 24));
}

#[test]
fn snapshot_merge_popup_arg_mode() {
    let test_repo = TestRepo::new();
    let mut model = create_command_popup_model(
        &test_repo,
        PopupContentCommand::Merge(MergePopupState { in_progress: false }),
    );
    model.arg_mode = true;
    model.arguments = Some(Arguments::merge_args(HashSet::from([
        MergeArgument::FfOnly,
    ])));
    // Argument mode only changes styling (key highlights, selected flags), so
    // snapshot the styled buffer instead of the plain-text frame.
    assert_frame_snapshot!(render_to_styled_string(&model, 80, 24));
}

#[test]
fn snapshot_merge_popup_with_strategy() {
    let test_repo = TestRepo::new();
    let mut model = create_command_popup_model(
        &test_repo,
        PopupContentCommand::Merge(MergePopupState { in_progress: false }),
    );
    model.arguments = Some(Arguments::MergeArguments {
        args: HashSet::new(),
        strategy: Some("recursive".to_string()),
        diff_algorithm: None,
        gpg_sign: None,
    });
    // The selected strategy is rendered as the flag value (--strategy=<value>)
    assert_frame_snapshot!(render_to_string(&model, 80, 24));
}

#[test]
fn snapshot_merge_popup_ignore_all_space_selected() {
    let test_repo = TestRepo::new();
    let mut model = create_command_popup_model(
        &test_repo,
        PopupContentCommand::Merge(MergePopupState { in_progress: false }),
    );
    model.arguments = Some(Arguments::merge_args(HashSet::from([
        MergeArgument::IgnoreAllSpace,
    ])));
    // The -w flag is rendered as active (-Xignore-all-space) below -b.
    assert_frame_snapshot!(render_to_string(&model, 80, 24));
}

#[test]
fn snapshot_merge_strategy_select_popup() {
    let test_repo = TestRepo::new();
    let mut model = create_command_popup_model(
        &test_repo,
        PopupContentCommand::Merge(MergePopupState { in_progress: false }),
    );
    update(&mut model, Message::ShowMergeStrategySelect);
    assert_frame_snapshot!(render_to_string(&model, 80, 24));
}

#[test]
fn snapshot_merge_popup_with_diff_algorithm() {
    let test_repo = TestRepo::new();
    let mut model = create_command_popup_model(
        &test_repo,
        PopupContentCommand::Merge(MergePopupState { in_progress: false }),
    );
    model.arguments = Some(Arguments::MergeArguments {
        args: HashSet::new(),
        strategy: None,
        diff_algorithm: Some("histogram".to_string()),
        gpg_sign: None,
    });
    // The selected algorithm is rendered as the flag value
    // (-Xdiff-algorithm=<value>) on the -A line.
    assert_frame_snapshot!(render_to_string(&model, 80, 24));
}

#[test]
fn snapshot_merge_diff_algorithm_select_popup() {
    let test_repo = TestRepo::new();
    let mut model = create_command_popup_model(
        &test_repo,
        PopupContentCommand::Merge(MergePopupState { in_progress: false }),
    );
    update(&mut model, Message::ShowMergeDiffAlgorithmSelect);
    assert_frame_snapshot!(render_to_string(&model, 80, 24));
}

#[test]
fn snapshot_merge_popup_with_gpg_sign() {
    let test_repo = TestRepo::new();
    let mut model = create_command_popup_model(
        &test_repo,
        PopupContentCommand::Merge(MergePopupState { in_progress: false }),
    );
    model.arguments = Some(Arguments::MergeArguments {
        args: HashSet::new(),
        strategy: None,
        diff_algorithm: None,
        gpg_sign: Some("ABCD1234".to_string()),
    });
    // The selected key is rendered as the flag value (--gpg-sign=<keyid>)
    // on the -S line.
    assert_frame_snapshot!(render_to_string(&model, 80, 24));
}

#[test]
fn snapshot_apply_popup() {
    let test_repo = TestRepo::new();
    let model = create_command_popup_model(
        &test_repo,
        PopupContentCommand::Apply(ApplyPopupState {
            in_progress: false,
            selected_commits: vec!["1234567890abcdef1234567890abcdef12345678".to_string()],
        }),
    );
    assert_frame_snapshot!(render_to_string(&model, 80, 24));
}

#[test]
fn snapshot_tag_popup() {
    let test_repo = TestRepo::new();
    let model = create_command_popup_model(&test_repo, PopupContentCommand::Tag);
    assert_frame_snapshot!(render_to_string(&model, 80, 24));
}

#[test]
fn snapshot_tag_release_input_popup() {
    let test_repo = TestRepo::new();
    let mut model = create_snapshot_model(&test_repo);
    model.popup = Some(PopupContent::Input(InputPopupState::with_text(
        InputContext::TagRelease {
            previous: Some("v1.0.0".to_string()),
        },
        "v1.0.0",
    )));
    assert_frame_snapshot!(render_to_string(&model, 80, 24));
}

#[test]
fn snapshot_tag_popup_with_force_argument() {
    let test_repo = TestRepo::new();
    let mut model = create_command_popup_model(&test_repo, PopupContentCommand::Tag);
    model.arg_mode = true;
    model.arguments = Some(Arguments::tag_args(HashSet::from([TagArgument::Force])));
    // Argument mode only changes styling (key highlights, selected flags), so
    // snapshot the styled buffer instead of the plain-text frame.
    assert_frame_snapshot!(render_to_styled_string(&model, 80, 24));
}

#[test]
fn snapshot_tag_popup_with_sign_as_value() {
    let test_repo = TestRepo::new();
    let mut model = create_command_popup_model(&test_repo, PopupContentCommand::Tag);
    model.arguments = Some(Arguments::TagArguments {
        args: HashSet::new(),
        local_user: Some("ABCD1234".to_string()),
    });
    assert_frame_snapshot!(render_to_string(&model, 80, 24));
}

#[test]
fn snapshot_select_popup() {
    let test_repo = TestRepo::new();
    let mut state = SelectPopupState::new(
        "Checkout".to_string(),
        vec![
            "main".to_string(),
            "feature/auth".to_string(),
            "feature/ui".to_string(),
            "develop".to_string(),
        ],
        OnSelect::CheckoutBranch,
    );
    state.input = InputField::from_text("feature");
    state.update_filter();
    let model = create_command_popup_model(&test_repo, PopupContentCommand::Select(state));
    assert_frame_snapshot!(render_to_string(&model, 80, 24));
}

// ── Other popups ──────────────────────────────────────────────────────────────

#[test]
fn snapshot_error_popup() {
    let test_repo = TestRepo::new();
    let mut model = create_snapshot_model(&test_repo);
    model.popup = Some(PopupContent::Error {
        message: "Failed to push: remote rejected".to_string(),
    });
    assert_frame_snapshot!(render_to_string(&model, 80, 24));
}

#[test]
fn snapshot_error_popup_short_message() {
    // A message narrower than the dismiss hint: the popup must still be wide
    // enough for "Press Enter or Esc to dismiss" not to be truncated.
    let test_repo = TestRepo::new();
    let mut model = create_snapshot_model(&test_repo);
    model.popup = Some(PopupContent::Error {
        message: "No tags found".to_string(),
    });
    assert_frame_snapshot!(render_to_string(&model, 80, 24));
}

#[test]
fn snapshot_merge_conflict_error_popup() {
    let test_repo = TestRepo::new();
    let mut model = create_snapshot_model(&test_repo);
    model.popup = Some(PopupContent::Error {
        message: "Merge of 'feature' stopped due to conflicts:\n\n  base.txt\n\nResolve the conflicts, then continue or abort from the merge popup (m).".to_string(),
    });
    assert_frame_snapshot!(render_to_string(&model, 80, 24));
}

#[test]
fn snapshot_confirm_popup() {
    let test_repo = TestRepo::new();
    let mut model = create_snapshot_model(&test_repo);
    model.popup = Some(PopupContent::Confirm(ConfirmPopupState {
        message: "Delete branch feature/auth?".to_string(),
        on_confirm: ConfirmAction::DeleteBranch("feature/auth".to_string()),
    }));
    assert_frame_snapshot!(render_to_string(&model, 80, 24));
}

#[test]
fn snapshot_credential_popup() {
    let test_repo = TestRepo::new();
    let mut model = create_snapshot_model(&test_repo);
    model.popup = Some(PopupContent::Credential(CredentialPopupState {
        credential_type: CredentialType::Password,
        input: InputField::from_text("hunter2"),
    }));
    assert_frame_snapshot!(render_to_string(&model, 80, 24));
}

#[test]
fn snapshot_input_popup() {
    let test_repo = TestRepo::new();
    let mut model = create_snapshot_model(&test_repo);
    model.popup = Some(PopupContent::Input(InputPopupState {
        input: InputField::from_text("feature/new-thing"),
        context: InputContext::CreateNewBranch {
            starting_point: "main".to_string(),
            checkout: true,
        },
    }));
    assert_frame_snapshot!(render_to_string(&model, 80, 24));
}

#[test]
fn snapshot_input_popup_mid_text_cursor() {
    use magi::model::EditOp;

    let test_repo = TestRepo::new();
    let mut model = create_snapshot_model(&test_repo);
    let mut input = InputField::from_text("feature/new-thing");
    // Move the cursor to just after "feature/" to exercise mid-text rendering
    input.apply(EditOp::MoveWordLeft);
    input.apply(EditOp::MoveWordLeft);
    model.popup = Some(PopupContent::Input(InputPopupState {
        input,
        context: InputContext::CreateNewBranch {
            starting_point: "main".to_string(),
            checkout: true,
        },
    }));
    assert_frame_snapshot!(render_to_string(&model, 80, 24));
}

// ── Preview virtualization ──────────────────────────────────────────────────────

/// Build a model in Preview mode holding `count` synthetic diff lines, as a
/// huge commit's `git show` output would produce. Used to exercise the
/// virtualized renderer, which must only build the lines inside the viewport
/// regardless of how large the buffer is.
fn preview_model_with_lines(test_repo: &TestRepo, count: usize) -> Model {
    let mut model = create_snapshot_model(test_repo);
    model.view_mode = ViewMode::Preview;
    model.ui_model.lines = (0..count)
        .map(|i| Line {
            content: LineContent::PreviewLine {
                content: format!("+added content on line {i}"),
                line_type: PreviewLineType::Addition,
            },
            section: None,
        })
        .collect();
    model.ui_model.viewport_height = 22;
    model
}

#[test]
fn snapshot_preview_large_diff_top() {
    let test_repo = TestRepo::new();
    let model = preview_model_with_lines(&test_repo, 50_000);
    // At the top of a 50k-line diff we should see the first lines.
    assert_frame_snapshot!(render_to_string(&model, 80, 24));
}

#[test]
fn snapshot_preview_large_diff_scrolled() {
    let test_repo = TestRepo::new();
    let mut model = preview_model_with_lines(&test_repo, 50_000);
    // Scrolled deep into the diff: the window must start at the scroll offset,
    // not the top of the buffer.
    model.ui_model.scroll_offset = 25_000;
    model.ui_model.cursor_position = 25_000;
    assert_frame_snapshot!(render_to_string(&model, 80, 24));
}

/// Regression test for the virtualized renderer: scrolling deep into a very
/// large diff must render the window at the scroll offset and nothing from the
/// far ends of the buffer. Before virtualization the whole buffer was rebuilt
/// every frame, which is what made browsing huge commits laggy.
#[test]
fn preview_large_diff_renders_only_visible_window() {
    let test_repo = TestRepo::new();
    let mut model = preview_model_with_lines(&test_repo, 50_000);
    model.ui_model.scroll_offset = 25_000;
    model.ui_model.cursor_position = 25_000;

    let frame = render_to_string(&model, 80, 24);

    // The line at the scroll offset is visible...
    assert!(
        frame.contains("line 25000"),
        "expected the line at the scroll offset to be visible:\n{frame}"
    );
    // ...while lines from the top and bottom of the buffer are not.
    assert!(
        !frame.contains("line 0 "),
        "did not expect the top of the buffer to render when scrolled:\n{frame}"
    );
    assert!(
        !frame.contains("line 49999"),
        "did not expect the bottom of the buffer to render when scrolled:\n{frame}"
    );
}
