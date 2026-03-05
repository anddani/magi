use magi::git::test_repo::TestRepo;
use magi::model::{RunningState, SectionType};
use magi::msg::Message;
use magi::msg::update::update;
use magi::msg::util::visible_lines_between;
use std::collections::HashSet;

mod utils;

use crate::utils::{create_model_from_test_repo, create_section_lines, create_test_model};

#[test]
fn test_refresh_message() {
    // TestRepo must stay alive so the workdir exists when Refresh reads from git
    let test_repo = TestRepo::new();
    let mut model = create_model_from_test_repo(&test_repo);

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
