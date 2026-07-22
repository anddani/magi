use magi::model::InputField;
use std::collections::HashSet;

use magi::{
    git::credential::CredentialType,
    model::{
        LineContent,
        arguments::{Arguments, PushArgument, TagArgument},
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
/// boundaries) so frames containing commits stay deterministic.
macro_rules! assert_frame_snapshot {
    ($frame:expr) => {
        insta::with_settings!({filters => vec![(r"\b[0-9a-f]{7,40}\b", "[hash]")]}, {
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
fn snapshot_stash_popup() {
    let test_repo = TestRepo::new();
    let model = create_command_popup_model(&test_repo, PopupContentCommand::Stash);
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
        }),
    );
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
    model.arguments = Some(Arguments::TagArguments(HashSet::from([TagArgument::Force])));
    // Argument mode only changes styling (key highlights, selected flags), so
    // snapshot the styled buffer instead of the plain-text frame.
    assert_frame_snapshot!(render_to_styled_string(&model, 80, 24));
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
