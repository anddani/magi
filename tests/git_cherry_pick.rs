use magi::{
    git::cherry_pick::get_cherry_picking_lines,
    git::test_repo::TestRepo,
    model::{LineContent, SectionType},
};
use std::fs;

fn git_dir(test_repo: &TestRepo) -> std::path::PathBuf {
    test_repo.repo_path().join(".git")
}

/// Returns the work directory (same as repo path for a standard repo).
fn workdir(test_repo: &TestRepo) -> std::path::PathBuf {
    test_repo.repo_path().to_path_buf()
}

// ── No cherry-pick in progress ────────────────────────────────────────────────

#[test]
fn test_no_cherry_pick_in_progress_returns_empty() {
    let test_repo = TestRepo::new();
    test_repo
        .write_file_content("file.txt", "hello")
        .stage_files(&["file.txt"])
        .commit("Initial commit");

    let lines = get_cherry_picking_lines(&workdir(&test_repo)).unwrap();
    assert!(lines.is_empty());
}

// ── CHERRY_PICK_HEAD present ──────────────────────────────────────────────────

#[test]
fn test_cherry_pick_head_produces_section_header() {
    let test_repo = TestRepo::new();
    test_repo
        .write_file_content("file.txt", "hello")
        .stage_files(&["file.txt"])
        .commit("Initial commit");

    let wd = workdir(&test_repo);
    let gd = git_dir(&test_repo);
    fs::write(gd.join("CHERRY_PICK_HEAD"), "abc1234567890\n").unwrap();

    let lines = get_cherry_picking_lines(&wd).unwrap();
    assert!(!lines.is_empty());

    // First line must be the section header
    match &lines[0].content {
        LineContent::SectionHeader { title, .. } => {
            assert_eq!(title, "Cherry Picking");
        }
        other => panic!("Expected SectionHeader, got {:?}", other),
    }
    assert_eq!(lines[0].section, Some(SectionType::CherryPicking));
}

#[test]
fn test_cherry_pick_head_produces_current_entry() {
    let test_repo = TestRepo::new();
    test_repo
        .write_file_content("file.txt", "hello")
        .stage_files(&["file.txt"])
        .commit("Initial commit");

    let wd = workdir(&test_repo);
    let gd = git_dir(&test_repo);
    // Write a fake (short) hash — read_commit_message will return empty for it
    fs::write(gd.join("CHERRY_PICK_HEAD"), "abc1234\n").unwrap();

    let lines = get_cherry_picking_lines(&wd).unwrap();

    // Should have header + one CherryPickingEntry
    assert_eq!(lines.len(), 2);
    match &lines[1].content {
        LineContent::CherryPickingEntry {
            hash, is_current, ..
        } => {
            assert_eq!(hash, "abc1234");
            assert!(*is_current, "The CHERRY_PICK_HEAD entry must be is_current");
        }
        other => panic!("Expected CherryPickingEntry, got {:?}", other),
    }
    assert_eq!(lines[1].section, Some(SectionType::CherryPicking));
}

// ── Sequencer todo present ────────────────────────────────────────────────────

#[test]
fn test_sequencer_todo_produces_pending_entries() {
    let test_repo = TestRepo::new();
    test_repo
        .write_file_content("file.txt", "hello")
        .stage_files(&["file.txt"])
        .commit("Initial commit");

    let wd = workdir(&test_repo);
    let gd = git_dir(&test_repo);
    let sequencer = gd.join("sequencer");
    fs::create_dir_all(&sequencer).unwrap();
    fs::write(
        sequencer.join("todo"),
        "pick deadbeef # First pending\npick cafebabe # Second pending\n",
    )
    .unwrap();

    let lines = get_cherry_picking_lines(&wd).unwrap();

    // Header + 2 pending entries
    assert_eq!(lines.len(), 3);

    match &lines[1].content {
        LineContent::CherryPickingEntry {
            hash,
            message,
            is_current,
        } => {
            assert_eq!(hash, "deadbee");
            assert_eq!(message, "First pending");
            assert!(!is_current);
        }
        other => panic!("Expected CherryPickingEntry, got {:?}", other),
    }

    match &lines[2].content {
        LineContent::CherryPickingEntry {
            hash,
            message,
            is_current,
        } => {
            assert_eq!(hash, "cafebab");
            assert_eq!(message, "Second pending");
            assert!(!is_current);
        }
        other => panic!("Expected CherryPickingEntry, got {:?}", other),
    }
}

#[test]
fn test_cherry_pick_head_and_todo_produces_both_entries() {
    let test_repo = TestRepo::new();
    test_repo
        .write_file_content("file.txt", "hello")
        .stage_files(&["file.txt"])
        .commit("Initial commit");

    let wd = workdir(&test_repo);
    let gd = git_dir(&test_repo);
    fs::write(gd.join("CHERRY_PICK_HEAD"), "abc1234\n").unwrap();

    let sequencer = gd.join("sequencer");
    fs::create_dir_all(&sequencer).unwrap();
    fs::write(sequencer.join("todo"), "pick deadbeef # Pending commit\n").unwrap();

    let lines = get_cherry_picking_lines(&wd).unwrap();

    // Header + current (CHERRY_PICK_HEAD) + pending (todo)
    assert_eq!(lines.len(), 3);

    match &lines[1].content {
        LineContent::CherryPickingEntry { is_current, .. } => assert!(*is_current),
        other => panic!("Expected CherryPickingEntry, got {:?}", other),
    }
    match &lines[2].content {
        LineContent::CherryPickingEntry { is_current, .. } => assert!(!is_current),
        other => panic!("Expected CherryPickingEntry, got {:?}", other),
    }
}

#[test]
fn test_only_header_returns_empty() {
    // If we somehow get into a state where there's no CHERRY_PICK_HEAD and the
    // todo file doesn't start with "pick", we should get no section at all.
    let test_repo = TestRepo::new();
    test_repo
        .write_file_content("file.txt", "hello")
        .stage_files(&["file.txt"])
        .commit("Initial commit");

    let wd = workdir(&test_repo);
    let gd = git_dir(&test_repo);
    let sequencer = gd.join("sequencer");
    fs::create_dir_all(&sequencer).unwrap();
    fs::write(sequencer.join("todo"), "revert abc1234 # some revert\n").unwrap();

    let lines = get_cherry_picking_lines(&wd).unwrap();
    assert!(lines.is_empty());
}
