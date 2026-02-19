use magi::config::Theme;
use magi::git::GitInfo;
use magi::git::test_repo::TestRepo;
use magi::model::{Model, RunningState, SectionType, UiModel, ViewMode};
use magi::msg::Message;
use magi::msg::update::update;
use magi::msg::util::visible_lines_between;
use std::collections::HashSet;

mod utils;

use crate::utils::{create_section_lines, create_test_model};

#[test]
fn test_refresh_message() {
    // This test needs the TestRepo to stay alive since Refresh reads from the git repo
    let test_repo = TestRepo::new();
    let repo_path = test_repo.repo.workdir().unwrap();
    let git_info = GitInfo::new_from_path(repo_path).unwrap();
    let lines = git_info.get_lines().unwrap();

    let workdir = repo_path.to_path_buf();
    let mut model = Model {
        git_info,
        workdir,
        running_state: RunningState::Running,
        ui_model: UiModel {
            lines,
            ..Default::default()
        },
        theme: Theme::default(),
        popup: None,
        toast: None,
        select_result: None,
        select_context: None,
        pty_state: None,
        arg_mode: false,
        pending_g: false,
        arguments: None,
        open_pr_branch: None,
        view_mode: ViewMode::Status,
    };

    // Clear the lines to simulate outdated state
    model.ui_model.lines.clear();
    assert!(model.ui_model.lines.is_empty());

    // Send refresh message
    update(&mut model, Message::Refresh);

    // Verify that lines were refreshed
    assert!(!model.ui_model.lines.is_empty());
    assert_eq!(model.running_state, RunningState::Running);
}

#[test]
fn test_quit_message() {
    let mut model = create_test_model();

    // Send quit message
    update(&mut model, Message::Quit);

    // Verify that running state changed to Done
    assert_eq!(model.running_state, RunningState::Done);
}

#[test]
fn test_visible_lines_between() {
    let lines = create_section_lines();
    let mut collapsed = HashSet::new();

    // No collapsed sections: all lines visible
    assert_eq!(visible_lines_between(&lines, 0, 3, &collapsed), 3);
    assert_eq!(visible_lines_between(&lines, 0, 7, &collapsed), 7);

    // Collapse UntrackedFiles section - hides lines 1, 2
    collapsed.insert(SectionType::UntrackedFiles);
    // Lines 0-3: line 0 visible, lines 1-2 hidden, so only 1 visible
    assert_eq!(visible_lines_between(&lines, 0, 3, &collapsed), 1);
    // Lines 0-7: lines 0, 3, 4, 5, 6 visible = 5
    assert_eq!(visible_lines_between(&lines, 0, 7, &collapsed), 5);
}
