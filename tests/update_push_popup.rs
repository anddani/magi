use std::collections::HashSet;

use magi::model::Model;
use magi::model::arguments::{Argument, Arguments, PushArgument};
use magi::model::popup::{
    PopupContent, PopupContentCommand, PushPopupState, SelectContext, SelectPopupState,
};
use magi::msg::update::update;
use magi::msg::{Message, PushCommand, SelectPopup};

use crate::utils::create_test_model;

mod utils;

fn create_push_popup_model() -> Model {
    let mut model = create_test_model();
    model.ui_model.viewport_height = 20;

    // Set up push popup state
    model.popup = Some(PopupContent::Command(PopupContentCommand::Push(
        PushPopupState {
            upstream: None,
            push_remote: None,
            sole_remote: None,
        },
    )));

    model
}

#[test]
fn test_push_enter_arg_mode() {
    let mut model = create_push_popup_model();

    // Verify arg_mode starts false
    assert!(!model.arg_mode);

    // Enter arg mode
    update(&mut model, Message::EnterArgMode);

    // Verify arg_mode is now true
    assert!(model.arg_mode);
}

#[test]
fn test_push_exit_arg_mode() {
    let mut model = create_push_popup_model();

    // Set arg_mode to true first
    model.arg_mode = true;

    // Exit arg mode
    update(&mut model, Message::ExitArgMode);

    // Verify arg_mode is now false
    assert!(!model.arg_mode);
}

#[test]
fn test_push_toggle_force_with_lease_enables() {
    let mut model = create_push_popup_model();

    // Set arg_mode to true first (as would happen in real usage)
    model.arg_mode = true;
    assert!(model.arguments.is_none());

    // Toggle force_with_lease
    update(
        &mut model,
        Message::ToggleArgument(Argument::Push(PushArgument::ForceWithLease)),
    );

    // Verify force_with_lease is now enabled and arg_mode is false
    match &model.arguments {
        Some(Arguments::PushArguments(args)) => {
            assert!(args.contains(&PushArgument::ForceWithLease));
        }
        _ => panic!("Expected PushArguments"),
    }
    assert!(!model.arg_mode); // Should exit arg mode after toggle
}

#[test]
fn test_push_toggle_force_with_lease_disables() {
    let mut model = create_push_popup_model();

    // Set force_with_lease to true and arg_mode to true
    model.arg_mode = true;
    let mut args = HashSet::new();
    args.insert(PushArgument::ForceWithLease);
    model.arguments = Some(Arguments::PushArguments(args));

    // Toggle force_with_lease
    update(
        &mut model,
        Message::ToggleArgument(Argument::Push(PushArgument::ForceWithLease)),
    );

    // Verify force_with_lease is now disabled and arg_mode is false
    match &model.arguments {
        Some(Arguments::PushArguments(args)) => {
            assert!(!args.contains(&PushArgument::ForceWithLease));
        }
        _ => panic!("Expected PushArguments"),
    }
    assert!(!model.arg_mode); // Should exit arg mode after toggle
}

// ── ShowSelectPopup::PushElsewhere shows select popup ─────────────────────────

#[test]
fn test_push_elsewhere_key_shows_select_popup() {
    use crate::utils::create_model_from_test_repo;
    use magi::git::test_repo::TestRepo;

    let test_repo = TestRepo::new();
    test_repo
        .write_file_content("file.txt", "content")
        .stage_files(&["file.txt"])
        .commit("Initial commit");

    let mut model = create_model_from_test_repo(&test_repo);

    // No remotes → should show an error popup (no remote branches to select)
    let result = update(
        &mut model,
        Message::ShowSelectPopup(SelectPopup::PushElsewhere),
    );

    assert_eq!(result, None);
    assert!(matches!(model.popup, Some(PopupContent::Error { .. })));
}

// ── SelectContext::PushElsewhere routes to PushElsewhere message ──────────────

#[test]
fn test_push_elsewhere_select_routes_to_push_message() {
    use magi::msg::SelectMessage;

    let mut model = create_test_model();

    // Simulate the state after the user has been shown the remote-branch picker
    // and "origin/main" is the selected item.
    model.select_context = Some(SelectContext::PushElsewhere);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Select(
        SelectPopupState::new(
            "Push to".to_string(),
            vec!["origin/main".to_string(), "origin/dev".to_string()],
        ),
    )));

    // Confirm the selection (first item "origin/main" is selected by default)
    let result = update(&mut model, Message::Select(SelectMessage::Confirm));

    assert_eq!(
        result,
        Some(Message::Push(PushCommand::PushElsewhere(
            "origin/main".to_string()
        )))
    );

    // Popup should be dismissed and context consumed
    assert!(model.popup.is_none());
    assert!(model.select_context.is_none());
}

// ── PushOtherBranchPick routes to PushOtherBranchTarget select ────────────────

#[test]
fn test_push_other_branch_pick_routes_to_target_select() {
    use magi::msg::SelectMessage;

    let mut model = create_test_model();

    // Simulate the state after the user has been shown the local branch picker
    // and "feature" is the selected item.
    model.select_context = Some(SelectContext::PushOtherBranchPick);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Select(
        SelectPopupState::new(
            "Push branch".to_string(),
            vec!["feature".to_string(), "main".to_string()],
        ),
    )));

    // Confirm the selection (first item "feature" is selected by default)
    let result = update(&mut model, Message::Select(SelectMessage::Confirm));

    assert_eq!(
        result,
        Some(Message::ShowSelectPopup(
            SelectPopup::PushOtherBranchTarget("feature".to_string())
        ))
    );

    // Popup should be dismissed and context consumed
    assert!(model.popup.is_none());
    assert!(model.select_context.is_none());
}

// ── PushOtherBranchTarget routes to Push(PushOtherBranch) ────────────────────

#[test]
fn test_push_other_branch_target_routes_to_push_message() {
    use magi::msg::SelectMessage;

    let mut model = create_test_model();

    // Simulate the state after the user has been shown the remote branch picker
    // and "origin/main" is the selected item.
    model.select_context = Some(SelectContext::PushOtherBranchTarget("feature".to_string()));
    model.popup = Some(PopupContent::Command(PopupContentCommand::Select(
        SelectPopupState::new(
            "Push to".to_string(),
            vec!["origin/main".to_string(), "origin/dev".to_string()],
        ),
    )));

    // Confirm the selection (first item "origin/main" is selected by default)
    let result = update(&mut model, Message::Select(SelectMessage::Confirm));

    assert_eq!(
        result,
        Some(Message::Push(PushCommand::PushOtherBranch {
            local: "feature".to_string(),
            remote: "origin/main".to_string(),
        }))
    );

    // Popup should be dismissed and context consumed
    assert!(model.popup.is_none());
    assert!(model.select_context.is_none());
}

// ── PushRefspecRemotePick routes to ShowPushRefspecInput ──────────────────────

#[test]
fn test_push_refspec_remote_pick_routes_to_input() {
    use magi::msg::SelectMessage;

    let mut model = create_test_model();

    model.select_context = Some(SelectContext::PushRefspecRemotePick);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Select(
        SelectPopupState::new(
            "Push to remote".to_string(),
            vec!["origin".to_string(), "upstream".to_string()],
        ),
    )));

    let result = update(&mut model, Message::Select(SelectMessage::Confirm));

    assert_eq!(
        result,
        Some(Message::ShowPushRefspecInput("origin".to_string()))
    );
    assert!(model.popup.is_none());
    assert!(model.select_context.is_none());
}

// ── ShowPushRefspecInput opens an input popup ─────────────────────────────────

#[test]
fn test_show_push_refspec_input_opens_popup() {
    use magi::model::popup::InputContext;

    let mut model = create_test_model();

    let result = update(
        &mut model,
        Message::ShowPushRefspecInput("origin".to_string()),
    );

    assert_eq!(result, None);
    assert!(matches!(
        model.popup,
        Some(PopupContent::Input(ref s))
            if matches!(&s.context, InputContext::PushRefspec { remote } if remote == "origin")
    ));
}

// ── Confirming refspec input dispatches Push(PushRefspecs) ───────────────────

#[test]
fn test_push_refspec_input_confirm_dispatches_push() {
    use magi::model::popup::{InputContext, InputPopupState};
    use magi::msg::InputMessage;

    let mut model = create_test_model();
    model.popup = Some(PopupContent::Input(InputPopupState {
        input_text: "HEAD:refs/heads/main".to_string(),
        context: InputContext::PushRefspec {
            remote: "origin".to_string(),
        },
    }));

    let result = update(&mut model, Message::Input(InputMessage::Confirm));

    assert_eq!(
        result,
        Some(Message::Push(PushCommand::PushRefspecs {
            remote: "origin".to_string(),
            refspecs: "HEAD:refs/heads/main".to_string(),
        }))
    );
    assert!(model.popup.is_none());
}
