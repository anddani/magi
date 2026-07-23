use std::collections::HashSet;

use magi::{
    git::test_repo::TestRepo,
    model::arguments::{Arguments, CommitArgument},
    msg::{Message, update::update},
};

mod utils;
use utils::{create_model_from_test_repo, expect_error_popup};

/// Point the repo's gpg.program at a program that always fails, which is how
/// a missing or broken signing key behaves.
fn break_gpg(test_repo: &TestRepo) {
    std::process::Command::new("git")
        .args(["config", "gpg.program", "false"])
        .current_dir(test_repo.repo_path())
        .output()
        .unwrap();
}

fn gpg_sign_arguments() -> Option<Arguments> {
    Some(Arguments::CommitArguments(HashSet::from([
        CommitArgument::GpgSign,
    ])))
}

#[test]
fn test_commit_with_sign_and_broken_gpg_shows_error_popup() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file1.txt", "content1", "First commit");
    test_repo
        .write_file_content("file1.txt", "modified")
        .stage_files(&["file1.txt"]);
    break_gpg(&test_repo);

    let mut model = create_model_from_test_repo(&test_repo);
    model.arguments = gpg_sign_arguments();

    update(&mut model, Message::Commit);

    let message = expect_error_popup(&model);
    assert!(
        message.contains("Cannot sign commit"),
        "unexpected message: {message}"
    );
    // The signing check runs before the editor, so nothing was committed.
    assert!(test_repo.head_hash().len() >= 7);
}

#[test]
fn test_amend_with_sign_and_broken_gpg_shows_error_popup() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file1.txt", "content1", "First commit");
    break_gpg(&test_repo);

    let head_before = test_repo.head_hash();
    let mut model = create_model_from_test_repo(&test_repo);
    model.arguments = gpg_sign_arguments();

    update(&mut model, Message::Amend(vec!["--no-edit".to_string()]));

    let message = expect_error_popup(&model);
    assert!(
        message.contains("Cannot sign commit"),
        "unexpected message: {message}"
    );
    assert_eq!(test_repo.head_hash(), head_before);
}
