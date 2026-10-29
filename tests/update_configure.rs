mod utils;

use magi::{
    git::{
        config::{get_local_config, set_branch_upstream},
        test_repo::TestRepo,
    },
    model::{
        InputField,
        popup::{ConfigurePopupState, PopupContent, PopupContentCommand},
        select_popup::OnSelect,
    },
    msg::{ConfigAction, InputMessage, Message, update::update},
};
use utils::create_model_from_test_repo;

/// Opens the Configure popup scoped to `branch` on the given model.
fn open_configure(model: &mut magi::model::Model, branch: &str) {
    model.popup = Some(PopupContent::Command(PopupContentCommand::Configure(
        ConfigurePopupState {
            branch: branch.to_string(),
            pending_a: false,
        },
    )));
}

#[test]
fn show_configure_on_branch_opens_popup_for_current_branch() {
    let test_repo = TestRepo::new();
    let mut model = create_model_from_test_repo(&test_repo);

    let follow = update(
        &mut model,
        Message::ShowConfigurePopup { force_pick: false },
    );

    assert!(follow.is_none());
    match &model.popup {
        Some(PopupContent::Command(PopupContentCommand::Configure(state))) => {
            assert_eq!(state.branch, "main");
            assert!(!state.pending_a);
        }
        other => panic!("expected Configure popup, got {:?}", other),
    }
}

#[test]
fn show_configure_detached_head_shows_branch_picker() {
    let test_repo = TestRepo::new();
    test_repo.detach_head();
    let mut model = create_model_from_test_repo(&test_repo);

    let follow = update(
        &mut model,
        Message::ShowConfigurePopup { force_pick: false },
    );

    match follow {
        Some(Message::ShowSelectPopup(cfg)) => {
            assert_eq!(cfg.on_select, OnSelect::ConfigureBranch);
        }
        other => panic!("expected ShowSelectPopup, got {:?}", other),
    }
}

#[test]
fn show_configure_force_pick_shows_branch_picker_even_on_branch() {
    let test_repo = TestRepo::new();
    let mut model = create_model_from_test_repo(&test_repo);

    let follow = update(&mut model, Message::ShowConfigurePopup { force_pick: true });

    match follow {
        Some(Message::ShowSelectPopup(cfg)) => {
            assert_eq!(cfg.on_select, OnSelect::ConfigureBranch);
        }
        other => panic!("expected ShowSelectPopup, got {:?}", other),
    }
}

#[test]
fn cycle_branch_rebase_goes_unset_true_false_unset() {
    let test_repo = TestRepo::new();
    let path = test_repo.repo_path().to_path_buf();
    let mut model = create_model_from_test_repo(&test_repo);
    open_configure(&mut model, "main");

    update(
        &mut model,
        Message::ConfigAction(ConfigAction::CycleBranchRebase),
    );
    assert_eq!(
        get_local_config(&path, "branch.main.rebase"),
        Some("true".to_string())
    );

    update(
        &mut model,
        Message::ConfigAction(ConfigAction::CycleBranchRebase),
    );
    assert_eq!(
        get_local_config(&path, "branch.main.rebase"),
        Some("false".to_string())
    );

    update(
        &mut model,
        Message::ConfigAction(ConfigAction::CycleBranchRebase),
    );
    assert_eq!(get_local_config(&path, "branch.main.rebase"), None);

    // The popup stays open so it re-renders from the freshly written config.
    assert!(matches!(
        model.popup,
        Some(PopupContent::Command(PopupContentCommand::Configure(_)))
    ));
}

#[test]
fn cycle_autosetup_merge_cycles_global_variable() {
    let test_repo = TestRepo::new();
    let path = test_repo.repo_path().to_path_buf();
    let mut model = create_model_from_test_repo(&test_repo);
    open_configure(&mut model, "main");

    update(
        &mut model,
        Message::ConfigAction(ConfigAction::CycleAutoSetupMerge),
    );
    assert_eq!(
        get_local_config(&path, "branch.autoSetupMerge"),
        Some("always".to_string())
    );
}

#[test]
fn edit_description_sets_then_clears() {
    let test_repo = TestRepo::new();
    let path = test_repo.repo_path().to_path_buf();
    let mut model = create_model_from_test_repo(&test_repo);
    open_configure(&mut model, "main");

    // Open the description input, type text, confirm.
    update(
        &mut model,
        Message::ConfigAction(ConfigAction::EditDescription),
    );
    if let Some(PopupContent::Input(state)) = &mut model.popup {
        state.input = InputField::from_text("a feature branch");
    } else {
        panic!("expected input popup");
    }
    update(&mut model, Message::Input(InputMessage::Confirm));
    assert_eq!(
        get_local_config(&path, "branch.main.description"),
        Some("a feature branch".to_string())
    );

    // Reopen and confirm empty to clear it.
    update(
        &mut model,
        Message::ConfigAction(ConfigAction::EditDescription),
    );
    if let Some(PopupContent::Input(state)) = &mut model.popup {
        state.input = InputField::new();
    } else {
        panic!("expected input popup");
    }
    update(&mut model, Message::Input(InputMessage::Confirm));
    assert_eq!(get_local_config(&path, "branch.main.description"), None);

    // Back to the Configure popup after confirming.
    assert!(matches!(
        model.popup,
        Some(PopupContent::Command(PopupContentCommand::Configure(_)))
    ));
}

#[test]
fn toggle_upstream_unsets_when_set() {
    let test_repo = TestRepo::new();
    let path = test_repo.repo_path().to_path_buf();
    test_repo.create_branch("other");
    set_branch_upstream(&path, "main", "other").unwrap();
    assert!(get_local_config(&path, "branch.main.remote").is_some());

    let mut model = create_model_from_test_repo(&test_repo);
    open_configure(&mut model, "main");

    let follow = update(
        &mut model,
        Message::ConfigAction(ConfigAction::ToggleUpstream),
    );
    assert!(follow.is_none());
    assert_eq!(get_local_config(&path, "branch.main.remote"), None);
    assert_eq!(get_local_config(&path, "branch.main.merge"), None);
}

#[test]
fn toggle_upstream_opens_picker_when_unset() {
    let test_repo = TestRepo::new();
    let mut model = create_model_from_test_repo(&test_repo);
    open_configure(&mut model, "main");

    let follow = update(
        &mut model,
        Message::ConfigAction(ConfigAction::ToggleUpstream),
    );
    match follow {
        Some(Message::ShowSelectPopup(cfg)) => {
            assert!(matches!(cfg.on_select, OnSelect::ConfigSetUpstream { .. }));
        }
        other => panic!("expected ShowSelectPopup, got {:?}", other),
    }
}

#[test]
fn configure_branch_selection_opens_configure_for_picked_branch() {
    use magi::model::select_popup::SelectPopupState;
    use magi::msg::SelectMessage;

    let test_repo = TestRepo::new();
    test_repo.create_branch("feature");
    let mut model = create_model_from_test_repo(&test_repo);

    // Simulate the branch picker having resolved to "feature".
    let mut state = SelectPopupState::new(
        "Configure branch".to_string(),
        vec!["feature".to_string()],
        OnSelect::ConfigureBranch,
    );
    state.selected_index = 0;
    model.popup = Some(PopupContent::Command(PopupContentCommand::Select(state)));

    update(&mut model, Message::Select(SelectMessage::Confirm));

    match &model.popup {
        Some(PopupContent::Command(PopupContentCommand::Configure(s))) => {
            assert_eq!(s.branch, "feature");
        }
        other => panic!("expected Configure popup, got {:?}", other),
    }
}
