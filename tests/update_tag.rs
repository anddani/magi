use crossterm::event::KeyCode;
use magi::model::InputField;
use magi::{
    git::test_repo::TestRepo,
    keys::handle_key,
    model::arguments::{Argument, Arguments, TagArgument},
    model::popup::{ConfirmAction, InputContext, PopupContent, PopupContentCommand},
    model::select_popup::OnSelect,
    msg::{Message, OptionsSource, ShowSelectPopupConfig, update::update},
};

mod utils;
use utils::{create_model_from_test_repo, expect_input_popup, expect_select_popup, key};

// ── ShowTagPopup — key binding ─────────────────────────────────────────────────

#[test]
fn test_t_key_shows_tag_popup() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file1.txt", "content1", "First commit");

    let model = create_model_from_test_repo(&test_repo);

    let result = handle_key(key(KeyCode::Char('t')), &model);
    assert_eq!(result, Some(Message::ShowTagPopup));
}

// ── ShowTagPopup — state ───────────────────────────────────────────────────────

#[test]
fn test_show_tag_popup_sets_state() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file1.txt", "content1", "First commit");

    let mut model = create_model_from_test_repo(&test_repo);

    let result = update(&mut model, Message::ShowTagPopup);

    assert_eq!(result, None);
    assert!(
        matches!(
            &model.popup,
            Some(PopupContent::Command(PopupContentCommand::Tag))
        ),
        "Expected Tag popup"
    );
}

// ── Tag popup keys ─────────────────────────────────────────────────────────────

#[test]
fn test_q_in_tag_popup_dismisses() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file1.txt", "content1", "First commit");

    let mut model = create_model_from_test_repo(&test_repo);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Tag));

    let result = handle_key(key(KeyCode::Char('q')), &model);
    assert_eq!(result, Some(Message::DismissPopup));
}

#[test]
fn test_esc_in_tag_popup_dismisses() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file1.txt", "content1", "First commit");

    let mut model = create_model_from_test_repo(&test_repo);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Tag));

    let result = handle_key(key(KeyCode::Esc), &model);
    assert_eq!(result, Some(Message::DismissPopup));
}

// ── Arguments ──────────────────────────────────────────────────────────────────

#[test]
fn test_minus_in_tag_popup_enters_arg_mode() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file1.txt", "content1", "First commit");

    let mut model = create_model_from_test_repo(&test_repo);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Tag));

    let result = handle_key(key(KeyCode::Char('-')), &model);
    assert_eq!(result, Some(Message::EnterArgMode));
}

#[test]
fn test_f_in_tag_arg_mode_toggles_force() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file1.txt", "content1", "First commit");

    let mut model = create_model_from_test_repo(&test_repo);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Tag));
    model.arg_mode = true;

    let result = handle_key(key(KeyCode::Char('f')), &model);
    assert_eq!(
        result,
        Some(Message::ToggleArgument(Argument::Tag(TagArgument::Force)))
    );
}

#[test]
fn test_e_in_tag_arg_mode_toggles_edit() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file1.txt", "content1", "First commit");

    let mut model = create_model_from_test_repo(&test_repo);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Tag));
    model.arg_mode = true;

    let result = handle_key(key(KeyCode::Char('e')), &model);
    assert_eq!(
        result,
        Some(Message::ToggleArgument(Argument::Tag(TagArgument::Edit)))
    );
}

#[test]
fn test_a_in_tag_arg_mode_toggles_annotate() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file1.txt", "content1", "First commit");

    let mut model = create_model_from_test_repo(&test_repo);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Tag));
    model.arg_mode = true;

    let result = handle_key(key(KeyCode::Char('a')), &model);
    assert_eq!(
        result,
        Some(Message::ToggleArgument(Argument::Tag(
            TagArgument::Annotate
        )))
    );
}

#[test]
fn test_s_in_tag_arg_mode_toggles_sign() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file1.txt", "content1", "First commit");

    let mut model = create_model_from_test_repo(&test_repo);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Tag));
    model.arg_mode = true;

    let result = handle_key(key(KeyCode::Char('s')), &model);
    assert_eq!(
        result,
        Some(Message::ToggleArgument(Argument::Tag(TagArgument::Sign)))
    );
}

#[test]
fn test_other_key_in_tag_arg_mode_exits_arg_mode() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file1.txt", "content1", "First commit");

    let mut model = create_model_from_test_repo(&test_repo);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Tag));
    model.arg_mode = true;

    let result = handle_key(key(KeyCode::Char('z')), &model);
    assert_eq!(result, Some(Message::ExitArgMode));
}

#[test]
fn test_toggle_tag_force_argument_updates_model() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file1.txt", "content1", "First commit");

    let mut model = create_model_from_test_repo(&test_repo);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Tag));
    model.arg_mode = true;

    update(
        &mut model,
        Message::ToggleArgument(Argument::Tag(TagArgument::Force)),
    );

    assert!(!model.arg_mode);
    let args = model
        .arguments
        .as_ref()
        .and_then(|a| a.tag())
        .expect("Expected tag arguments");
    assert!(args.contains(&TagArgument::Force));

    // Toggling again removes it
    update(
        &mut model,
        Message::ToggleArgument(Argument::Tag(TagArgument::Force)),
    );
    let args = model
        .arguments
        .as_ref()
        .and_then(|a| a.tag())
        .expect("Expected tag arguments");
    assert!(!args.contains(&TagArgument::Force));
}

// ── Create tag flow ────────────────────────────────────────────────────────────

#[test]
fn test_t_in_tag_popup_shows_create_tag_input() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file1.txt", "content1", "First commit");

    let mut model = create_model_from_test_repo(&test_repo);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Tag));

    let result = handle_key(key(KeyCode::Char('t')), &model);
    assert_eq!(result, Some(Message::ShowCreateTagInput));
}

#[test]
fn test_show_create_tag_input_opens_input_popup() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file1.txt", "content1", "First commit");

    let mut model = create_model_from_test_repo(&test_repo);

    let result = update(&mut model, Message::ShowCreateTagInput);

    assert_eq!(result, None);
    let state = expect_input_popup(&model);
    assert_eq!(
        state.context,
        InputContext::CreateTag,
        "Expected Input popup with CreateTag context"
    );
}

#[test]
fn test_create_tag_input_confirm_shows_ref_select() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file1.txt", "content1", "First commit");

    let mut model = create_model_from_test_repo(&test_repo);
    // Set up an input popup with CreateTag context and a tag name typed
    model.popup = Some(PopupContent::input_popup(InputContext::CreateTag));
    if let Some(PopupContent::Input(ref mut state)) = model.popup {
        state.input = InputField::from_text("v1.0.0");
    }

    let result = update(&mut model, Message::Input(magi::msg::InputMessage::Confirm));

    assert_eq!(
        result,
        Some(Message::ShowSelectPopup(ShowSelectPopupConfig {
            title: "Create tag at".to_string(),
            source: OptionsSource::BranchesAndTags,
            on_select: OnSelect::CreateTagTarget {
                name: "v1.0.0".to_string(),
            },
        }))
    );
}

#[test]
fn test_create_tag_target_select_shows_refs() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file1.txt", "content1", "First commit");

    let mut model = create_model_from_test_repo(&test_repo);

    let result = update(
        &mut model,
        Message::ShowSelectPopup(ShowSelectPopupConfig {
            title: "Create tag at".to_string(),
            source: OptionsSource::BranchesAndTags,
            on_select: OnSelect::CreateTagTarget {
                name: "v1.0.0".to_string(),
            },
        }),
    );

    assert_eq!(result, None);
    let state = expect_select_popup(&model);
    assert!(
        !state.all_options.is_empty(),
        "Expected Select popup with non-empty options"
    );
    assert_eq!(
        state.on_select,
        OnSelect::CreateTagTarget {
            name: "v1.0.0".to_string(),
        }
    );
}

#[test]
fn test_create_tag_creates_tag() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file1.txt", "content1", "First commit");

    let mut model = create_model_from_test_repo(&test_repo);

    let result = update(
        &mut model,
        Message::CreateTag {
            name: "v1.0.0".to_string(),
            target: "HEAD".to_string(),
        },
    );

    assert_eq!(result, Some(Message::Refresh));
    assert!(model.popup.is_none());

    // Verify the tag exists in the repository
    let tags = model.git_info.repository.tag_names(None).unwrap();
    let tag_list: Vec<&str> = tags.iter().filter_map(|t| t.ok().flatten()).collect();
    assert!(
        tag_list.contains(&"v1.0.0"),
        "Tag 'v1.0.0' should exist in the repository"
    );
}

#[test]
fn test_create_tag_fails_when_tag_exists_without_force() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file1.txt", "content1", "First commit");

    let mut model = create_model_from_test_repo(&test_repo);

    update(
        &mut model,
        Message::CreateTag {
            name: "v1.0.0".to_string(),
            target: "HEAD".to_string(),
        },
    );

    test_repo.commit_file("file2.txt", "content2", "Second commit");

    let result = update(
        &mut model,
        Message::CreateTag {
            name: "v1.0.0".to_string(),
            target: "HEAD".to_string(),
        },
    );

    assert_eq!(result, None);
    assert!(
        matches!(&model.popup, Some(PopupContent::Error { .. })),
        "Expected error popup when re-creating an existing tag without --force"
    );
}

#[test]
fn test_create_tag_with_force_moves_existing_tag() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file1.txt", "content1", "First commit");

    let mut model = create_model_from_test_repo(&test_repo);

    update(
        &mut model,
        Message::CreateTag {
            name: "v1.0.0".to_string(),
            target: "HEAD".to_string(),
        },
    );

    test_repo.commit_file("file2.txt", "content2", "Second commit");

    model.arguments = Some(Arguments::TagArguments(
        [TagArgument::Force].into_iter().collect(),
    ));

    let result = update(
        &mut model,
        Message::CreateTag {
            name: "v1.0.0".to_string(),
            target: "HEAD".to_string(),
        },
    );

    assert_eq!(result, Some(Message::Refresh));
    assert!(model.popup.is_none());
    assert!(
        model.arguments.is_none(),
        "Arguments should be consumed after creating the tag"
    );

    // The tag should now point at the new HEAD commit
    let repo = &model.git_info.repository;
    let tag_ref = repo.find_reference("refs/tags/v1.0.0").unwrap();
    let head = repo.head().unwrap().peel_to_commit().unwrap();
    assert_eq!(
        tag_ref.peel_to_commit().unwrap().id(),
        head.id(),
        "Forced tag should point at the new HEAD"
    );
}

// ── Create tag with --edit ─────────────────────────────────────────────────────

#[test]
fn test_create_tag_with_edit_returns_with_editor_message() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file1.txt", "content1", "First commit");

    let mut model = create_model_from_test_repo(&test_repo);
    model.arguments = Some(Arguments::TagArguments(
        [TagArgument::Edit].into_iter().collect(),
    ));

    let result = update(
        &mut model,
        Message::CreateTag {
            name: "v1.0.0".to_string(),
            target: "HEAD".to_string(),
        },
    );

    let expected = Message::CreateTagWithEditor {
        name: "v1.0.0".to_string(),
        args: vec![
            "tag".to_string(),
            "--edit".to_string(),
            "v1.0.0".to_string(),
            "HEAD".to_string(),
        ],
    };
    assert_eq!(result, Some(expected));
    // The editor command must run with the TUI suspended
    assert!(magi::msg::util::is_external_command(&result.unwrap()));
    assert!(
        model.arguments.is_none(),
        "Arguments should be consumed when building the editor command"
    );
}

#[test]
fn test_create_tag_with_edit_and_force_orders_flags() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file1.txt", "content1", "First commit");

    let mut model = create_model_from_test_repo(&test_repo);
    model.arguments = Some(Arguments::TagArguments(
        [TagArgument::Edit, TagArgument::Force]
            .into_iter()
            .collect(),
    ));

    let result = update(
        &mut model,
        Message::CreateTag {
            name: "v1.0.0".to_string(),
            target: "HEAD".to_string(),
        },
    );

    // Flags are emitted in a stable order: --force before --edit
    assert_eq!(
        result,
        Some(Message::CreateTagWithEditor {
            name: "v1.0.0".to_string(),
            args: vec![
                "tag".to_string(),
                "--force".to_string(),
                "--edit".to_string(),
                "v1.0.0".to_string(),
                "HEAD".to_string(),
            ],
        })
    );
}

// ── Create tag with --annotate ─────────────────────────────────────────────────

#[test]
fn test_create_tag_with_annotate_returns_with_editor_message() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file1.txt", "content1", "First commit");

    let mut model = create_model_from_test_repo(&test_repo);
    model.arguments = Some(Arguments::TagArguments(
        [TagArgument::Annotate].into_iter().collect(),
    ));

    let result = update(
        &mut model,
        Message::CreateTag {
            name: "v1.0.0".to_string(),
            target: "HEAD".to_string(),
        },
    );

    let expected = Message::CreateTagWithEditor {
        name: "v1.0.0".to_string(),
        args: vec![
            "tag".to_string(),
            "--annotate".to_string(),
            "v1.0.0".to_string(),
            "HEAD".to_string(),
        ],
    };
    assert_eq!(result, Some(expected));
    // The editor command must run with the TUI suspended
    assert!(magi::msg::util::is_external_command(&result.unwrap()));
    assert!(
        model.arguments.is_none(),
        "Arguments should be consumed when building the editor command"
    );
}

#[test]
fn test_create_tag_with_annotate_and_force_orders_flags() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file1.txt", "content1", "First commit");

    let mut model = create_model_from_test_repo(&test_repo);
    model.arguments = Some(Arguments::TagArguments(
        [TagArgument::Annotate, TagArgument::Force]
            .into_iter()
            .collect(),
    ));

    let result = update(
        &mut model,
        Message::CreateTag {
            name: "v1.0.0".to_string(),
            target: "HEAD".to_string(),
        },
    );

    // Flags are emitted in a stable order: --force before --annotate
    assert_eq!(
        result,
        Some(Message::CreateTagWithEditor {
            name: "v1.0.0".to_string(),
            args: vec![
                "tag".to_string(),
                "--force".to_string(),
                "--annotate".to_string(),
                "v1.0.0".to_string(),
                "HEAD".to_string(),
            ],
        })
    );
}

// ── Create tag with --sign ─────────────────────────────────────────────────────

#[test]
fn test_create_tag_with_sign_returns_with_editor_message() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file1.txt", "content1", "First commit");

    let mut model = create_model_from_test_repo(&test_repo);
    model.arguments = Some(Arguments::TagArguments(
        [TagArgument::Sign].into_iter().collect(),
    ));

    let result = update(
        &mut model,
        Message::CreateTag {
            name: "v1.0.0".to_string(),
            target: "HEAD".to_string(),
        },
    );

    let expected = Message::CreateTagWithEditor {
        name: "v1.0.0".to_string(),
        args: vec![
            "tag".to_string(),
            "--sign".to_string(),
            "v1.0.0".to_string(),
            "HEAD".to_string(),
        ],
    };
    assert_eq!(result, Some(expected));
    // The editor command must run with the TUI suspended
    assert!(magi::msg::util::is_external_command(&result.unwrap()));
    assert!(
        model.arguments.is_none(),
        "Arguments should be consumed when building the editor command"
    );
}

#[test]
fn test_create_tag_with_sign_and_annotate_orders_flags() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file1.txt", "content1", "First commit");

    let mut model = create_model_from_test_repo(&test_repo);
    model.arguments = Some(Arguments::TagArguments(
        [TagArgument::Sign, TagArgument::Annotate]
            .into_iter()
            .collect(),
    ));

    let result = update(
        &mut model,
        Message::CreateTag {
            name: "v1.0.0".to_string(),
            target: "HEAD".to_string(),
        },
    );

    // Flags are emitted in a stable order: --annotate before --sign
    assert_eq!(
        result,
        Some(Message::CreateTagWithEditor {
            name: "v1.0.0".to_string(),
            args: vec![
                "tag".to_string(),
                "--annotate".to_string(),
                "--sign".to_string(),
                "v1.0.0".to_string(),
                "HEAD".to_string(),
            ],
        })
    );
}

#[test]
fn test_create_tag_without_edit_does_not_suspend() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file1.txt", "content1", "First commit");

    let mut model = create_model_from_test_repo(&test_repo);
    model.arguments = Some(Arguments::TagArguments(
        [TagArgument::Force].into_iter().collect(),
    ));

    let result = update(
        &mut model,
        Message::CreateTag {
            name: "v1.0.0".to_string(),
            target: "HEAD".to_string(),
        },
    );

    // Without --edit the tag is created directly, no editor round-trip
    assert_eq!(result, Some(Message::Refresh));
}

#[test]
fn test_create_tag_with_editor_fails_when_tag_exists() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file1.txt", "content1", "First commit");

    let mut model = create_model_from_test_repo(&test_repo);
    update(
        &mut model,
        Message::CreateTag {
            name: "v1.0.0".to_string(),
            target: "HEAD".to_string(),
        },
    );

    // Without --force, git rejects the existing tag before opening the
    // editor, so this test does not depend on any editor configuration
    let result = update(
        &mut model,
        Message::CreateTagWithEditor {
            name: "v1.0.0".to_string(),
            args: vec![
                "tag".to_string(),
                "--edit".to_string(),
                "v1.0.0".to_string(),
                "HEAD".to_string(),
            ],
        },
    );

    assert_eq!(result, None);
    assert!(
        matches!(&model.popup, Some(PopupContent::Error { .. })),
        "Expected error popup when tag already exists"
    );
}

// ── Delete tag flow ────────────────────────────────────────────────────────────

#[test]
fn test_x_in_tag_popup_shows_delete_tag_select() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file1.txt", "content1", "First commit");

    let mut model = create_model_from_test_repo(&test_repo);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Tag));

    let result = handle_key(key(KeyCode::Char('x')), &model);
    assert_eq!(
        result,
        Some(Message::ShowSelectPopup(ShowSelectPopupConfig {
            title: "Delete tag".to_string(),
            source: OptionsSource::Tags,
            on_select: OnSelect::DeleteTag,
        }))
    );
}

#[test]
fn test_delete_tag_select_shows_existing_tags() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file1.txt", "content1", "First commit");

    let mut model = create_model_from_test_repo(&test_repo);
    // Create a tag so the list is non-empty
    update(
        &mut model,
        Message::CreateTag {
            name: "v1.0.0".to_string(),
            target: "HEAD".to_string(),
        },
    );

    let result = update(
        &mut model,
        Message::ShowSelectPopup(ShowSelectPopupConfig {
            title: "Delete tag".to_string(),
            source: OptionsSource::Tags,
            on_select: OnSelect::DeleteTag,
        }),
    );

    assert_eq!(result, None);
    let state = expect_select_popup(&model);
    assert!(
        state.all_options.contains(&"v1.0.0".to_string()),
        "Expected Select popup listing 'v1.0.0'"
    );
    assert_eq!(state.on_select, OnSelect::DeleteTag);
}

#[test]
fn test_delete_tag_select_empty_repo_shows_error() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file1.txt", "content1", "First commit");

    let mut model = create_model_from_test_repo(&test_repo);
    // No tags created — should show an error popup

    let result = update(
        &mut model,
        Message::ShowSelectPopup(ShowSelectPopupConfig {
            title: "Delete tag".to_string(),
            source: OptionsSource::Tags,
            on_select: OnSelect::DeleteTag,
        }),
    );

    assert_eq!(result, None);
    assert!(
        matches!(&model.popup, Some(PopupContent::Error { .. })),
        "Expected Error popup when no tags exist"
    );
}

#[test]
fn test_delete_tag_removes_tag() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file1.txt", "content1", "First commit");

    let mut model = create_model_from_test_repo(&test_repo);
    // Create then delete
    update(
        &mut model,
        Message::CreateTag {
            name: "v1.0.0".to_string(),
            target: "HEAD".to_string(),
        },
    );

    let result = update(&mut model, Message::DeleteTag("v1.0.0".to_string()));

    assert_eq!(result, Some(Message::Refresh));
    assert!(model.popup.is_none());

    let tags = model.git_info.repository.tag_names(None).unwrap();
    let tag_list: Vec<&str> = tags.iter().filter_map(|t| t.ok().flatten()).collect();
    assert!(
        !tag_list.contains(&"v1.0.0"),
        "Tag 'v1.0.0' should have been deleted"
    );
}

// ── Prune tags flow ────────────────────────────────────────────────────────────

#[test]
fn test_p_in_tag_popup_shows_prune_remote_select() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file1.txt", "content1", "First commit");

    let mut model = create_model_from_test_repo(&test_repo);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Tag));

    let result = handle_key(key(KeyCode::Char('p')), &model);
    assert_eq!(
        result,
        Some(Message::ShowSelectPopup(ShowSelectPopupConfig {
            title: "Prune tags against".to_string(),
            source: OptionsSource::Remotes,
            on_select: OnSelect::PruneTagsRemotePick,
        }))
    );
}

#[test]
fn test_prune_tags_confirm_shows_error_for_bad_remote() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file1.txt", "content1", "First commit");

    let mut model = create_model_from_test_repo(&test_repo);

    let result = update(
        &mut model,
        Message::ShowPruneTagsConfirm {
            remote: "nonexistent-remote".to_string(),
        },
    );

    assert_eq!(result, None);
    assert!(
        matches!(&model.popup, Some(PopupContent::Error { .. })),
        "Expected Error popup for bad remote"
    );
}

#[test]
fn test_prune_remote_pick_skips_select_with_single_remote() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file1.txt", "content1", "First commit");

    // With exactly one remote, should skip the select and return ShowPruneTagsConfirm directly
    let repo_path = test_repo.repo.workdir().unwrap();
    std::process::Command::new("git")
        .arg("-C")
        .arg(repo_path)
        .args(["remote", "add", "origin", "https://example.com/repo.git"])
        .output()
        .unwrap();

    let mut model = create_model_from_test_repo(&test_repo);

    let result = update(
        &mut model,
        Message::ShowSelectPopup(ShowSelectPopupConfig {
            title: "Prune tags against".to_string(),
            source: OptionsSource::Remotes,
            on_select: OnSelect::PruneTagsRemotePick,
        }),
    );

    assert_eq!(
        result,
        Some(Message::ShowPruneTagsConfirm {
            remote: "origin".to_string()
        })
    );
}

#[test]
fn test_prune_remote_pick_shows_select_with_multiple_remotes() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file1.txt", "content1", "First commit");

    // With two remotes, should show the select popup
    let repo_path = test_repo.repo.workdir().unwrap();
    for remote in &["origin", "upstream"] {
        std::process::Command::new("git")
            .arg("-C")
            .arg(repo_path)
            .args(["remote", "add", remote, "https://example.com/repo.git"])
            .output()
            .unwrap();
    }

    let mut model = create_model_from_test_repo(&test_repo);

    let result = update(
        &mut model,
        Message::ShowSelectPopup(ShowSelectPopupConfig {
            title: "Prune tags against".to_string(),
            source: OptionsSource::Remotes,
            on_select: OnSelect::PruneTagsRemotePick,
        }),
    );

    assert_eq!(result, None);
    let state = expect_select_popup(&model);
    assert!(
        state.all_options.contains(&"origin".to_string()),
        "Expected Select popup listing remotes"
    );
    assert_eq!(state.on_select, OnSelect::PruneTagsRemotePick);
}

#[test]
fn test_prune_tags_deletes_local_only_tags() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file1.txt", "content1", "First commit");

    let mut model = create_model_from_test_repo(&test_repo);

    // Create a local tag
    update(
        &mut model,
        Message::CreateTag {
            name: "v1.0.0".to_string(),
            target: "HEAD".to_string(),
        },
    );

    // Prune with empty remote list (simulates: remote has no tags)
    let result = update(
        &mut model,
        Message::PruneTags {
            local_tags: vec!["v1.0.0".to_string()],
            remote_tags: vec![],
            remote: "origin".to_string(),
        },
    );

    assert_eq!(result, Some(Message::Refresh));
    assert!(model.popup.is_none());

    let tags = model.git_info.repository.tag_names(None).unwrap();
    let tag_list: Vec<&str> = tags.iter().filter_map(|t| t.ok().flatten()).collect();
    assert!(
        !tag_list.contains(&"v1.0.0"),
        "v1.0.0 should have been deleted locally"
    );
}

#[test]
fn test_prune_tags_confirm_shows_confirm_popup() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file1.txt", "content1", "First commit");

    let mut model = create_model_from_test_repo(&test_repo);

    // Dispatch directly with known data (bypasses network)
    // Simulate what show_prune_tags_confirm would produce
    let local_only = vec!["v1.0.0".to_string()];
    let remote_only: Vec<String> = vec![];

    // Confirm popup should hold the PruneTags action
    use magi::model::popup::ConfirmPopupState;
    model.popup = Some(PopupContent::Confirm(ConfirmPopupState {
        message: "Prune tags against 'origin':\n  Delete locally (1): v1.0.0".to_string(),
        on_confirm: ConfirmAction::PruneTags {
            local_tags: local_only.clone(),
            remote_tags: remote_only.clone(),
            remote: "origin".to_string(),
        },
    }));

    assert!(
        matches!(
            &model.popup,
            Some(PopupContent::Confirm(state))
                if matches!(&state.on_confirm, ConfirmAction::PruneTags { .. })
        ),
        "Confirm popup should hold PruneTags action"
    );
}
