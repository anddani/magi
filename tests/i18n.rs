use magi::i18n::{self, Language, Strings};

/// All `*_fmt` fields of `Strings`, used with `fmt1()` and required to contain
/// exactly one `{}` placeholder in every language. Locale completeness itself
/// is enforced by the compiler (every `Strings` field must be populated), but
/// nothing stops a translation from dropping the placeholder — this list does.
/// Keep in sync with the `*_fmt` fields in src/i18n.rs.
const FMT_FIELDS: [(&str, fn(&Strings) -> &'static str); 21] = [
    ("input_rename_branch_fmt", |s| s.input_rename_branch_fmt),
    ("help_version_fmt", |s| s.help_version_fmt),
    ("input_worktree_path_fmt", |s| s.input_worktree_path_fmt),
    ("input_worktree_move_path_fmt", |s| {
        s.input_worktree_move_path_fmt
    }),
    ("input_push_refspec_fmt", |s| s.input_push_refspec_fmt),
    ("input_fetch_refspec_fmt", |s| s.input_fetch_refspec_fmt),
    ("input_release_tag_fmt", |s| s.input_release_tag_fmt),
    ("input_stash_branch_fmt", |s| s.input_stash_branch_fmt),
    ("input_stash_branch_here_fmt", |s| {
        s.input_stash_branch_here_fmt
    }),
    ("completed_successfully_fmt", |s| {
        s.completed_successfully_fmt
    }),
    ("title_pick_rebase_subset_fmt", |s| {
        s.title_pick_rebase_subset_fmt
    }),
    ("title_fetch_branch_from_fmt", |s| {
        s.title_fetch_branch_from_fmt
    }),
    ("op_fetch_from_fmt", |s| s.op_fetch_from_fmt),
    ("op_fetch_refspecs_from_fmt", |s| {
        s.op_fetch_refspecs_from_fmt
    }),
    ("op_push_to_fmt", |s| s.op_push_to_fmt),
    ("op_push_refspecs_to_fmt", |s| s.op_push_refspecs_to_fmt),
    ("op_push_matching_branches_to_fmt", |s| {
        s.op_push_matching_branches_to_fmt
    }),
    ("op_push_tag_fmt", |s| s.op_push_tag_fmt),
    ("op_pull_from_fmt", |s| s.op_pull_from_fmt),
    ("op_rebase_onto_fmt", |s| s.op_rebase_onto_fmt),
    ("op_rebase_subset_onto_fmt", |s| s.op_rebase_subset_onto_fmt),
];

/// `*_fmt` fields that take exactly two `{}` placeholders via fmt2().
const FMT2_FIELDS: [(&str, fn(&Strings) -> &'static str); 1] =
    [("op_push_local_to_fmt", |s| s.op_push_local_to_fmt)];

#[test]
fn test_fmt2_fields_have_exactly_two_placeholders_in_all_languages() {
    for (lang_name, strings) in [("english", i18n::english()), ("swedish", i18n::swedish())] {
        for (field_name, get) in FMT2_FIELDS {
            let value = get(&strings);
            assert_eq!(
                value.matches("{}").count(),
                2,
                "{}::{} must contain exactly two {{}} placeholders, got: {:?}",
                lang_name,
                field_name,
                value
            );
        }
    }
}

#[test]
fn test_fmt_fields_have_exactly_one_placeholder_in_all_languages() {
    for (lang_name, strings) in [("english", i18n::english()), ("swedish", i18n::swedish())] {
        for (field_name, get) in FMT_FIELDS {
            let value = get(&strings);
            assert_eq!(
                value.matches("{}").count(),
                1,
                "{}::{} must contain exactly one {{}} placeholder, got: {:?}",
                lang_name,
                field_name,
                value
            );
        }
    }
}

#[test]
fn test_language_from_str_english() {
    assert!(matches!(Language::from_str("en"), Some(Language::English)));
    assert!(matches!(
        Language::from_str("english"),
        Some(Language::English)
    ));
}

#[test]
fn test_language_from_str_swedish() {
    assert!(matches!(Language::from_str("sv"), Some(Language::Swedish)));
    assert!(matches!(
        Language::from_str("swedish"),
        Some(Language::Swedish)
    ));
    assert!(matches!(
        Language::from_str("svenska"),
        Some(Language::Swedish)
    ));
    assert!(matches!(Language::from_str("SV"), Some(Language::Swedish)));
}

#[test]
fn test_language_from_str_unknown() {
    assert!(Language::from_str("de").is_none());
    assert!(Language::from_str("french").is_none());
    assert!(Language::from_str("").is_none());
}

#[test]
fn test_english_section_headers() {
    let s = i18n::english();
    assert_eq!(s.section_untracked_files, "Untracked files");
    assert_eq!(s.section_unstaged_changes, "Unstaged changes");
    assert_eq!(s.section_staged_changes, "Staged changes");
    assert_eq!(s.section_recent_commits, "Recent commits");
    assert_eq!(s.section_stashes, "Stashes");
    assert_eq!(s.section_rebasing, "Rebasing");
    assert_eq!(s.section_reverting, "Reverting");
    assert_eq!(s.section_cherry_picking, "Cherry Picking");
    assert_eq!(s.section_unpulled_from_prefix, "Unpulled from ");
}

#[test]
fn test_swedish_section_headers() {
    let s = i18n::swedish();
    assert_eq!(s.section_untracked_files, "Ospårade filer");
    assert_eq!(s.section_unstaged_changes, "Ej klarmarkerade ändringar");
    assert_eq!(s.section_staged_changes, "Klarmarkerade ändringar");
    assert_eq!(s.section_recent_commits, "Senaste förbindelser");
    assert_eq!(s.section_stashes, "Gömda ändringar");
    assert_eq!(s.section_rebasing, "Ympar");
    assert_eq!(s.section_reverting, "Återgår");
    assert_eq!(s.section_cherry_picking, "Plockar russin");
    assert_eq!(s.section_unpulled_from_prefix, "Ej ryckta från ");
}

#[test]
fn test_english_popup_titles() {
    let s = i18n::english();
    assert_eq!(s.popup_branch, "Branch");
    assert_eq!(s.popup_commit, "Commit");
    assert_eq!(s.popup_push, "Push");
    assert_eq!(s.popup_pull, "Pull");
    assert_eq!(s.popup_fetch, "Fetch");
    assert_eq!(s.popup_log, "Log");
    assert_eq!(s.popup_stash, "Stash");
    assert_eq!(s.popup_tag, "Tag");
    assert_eq!(s.popup_reset, "Reset");
    assert_eq!(s.popup_rebase, "Rebase");
    assert_eq!(s.popup_revert, "Revert");
    assert_eq!(s.popup_merge, "Merge");
    assert_eq!(s.popup_merging, "Merging");
    assert_eq!(s.popup_apply, "Apply");
    assert_eq!(s.popup_applying, "Applying");
    assert_eq!(s.popup_help, "Help");
    assert_eq!(s.popup_error, "Error");
    assert_eq!(s.popup_confirm, "Confirm");
}

#[test]
fn test_swedish_popup_titles() {
    let s = i18n::swedish();
    assert_eq!(s.popup_branch, "Grena");
    assert_eq!(s.popup_commit, "Förbinda");
    assert_eq!(s.popup_push, "Knuffa");
    assert_eq!(s.popup_pull, "Rycka");
    assert_eq!(s.popup_fetch, "Hämta");
    assert_eq!(s.popup_merge, "Sammanfoga");
    assert_eq!(s.popup_merging, "Sammanfogar");
    assert_eq!(s.popup_error, "Fel");
    assert_eq!(s.popup_help, "Hjälp");
}

#[test]
fn test_english_select_popup_titles() {
    let s = i18n::english();
    assert_eq!(s.title_checkout, "Checkout");
    assert_eq!(
        s.title_create_branch_starting_at,
        "Create branch starting at"
    );
    assert_eq!(s.title_push_to, "Push to");
    assert_eq!(s.title_push_to_push_remote, "Push to push remote");
    assert_eq!(s.title_pull_from, "Pull from");
    assert_eq!(s.title_fetch_from, "Fetch from");
    assert_eq!(s.title_merge_branch_no_commit, "Merge branch (no commit)");
    assert_eq!(s.title_rebase_subset_onto, "Rebase subset onto");
    assert_eq!(s.title_apply_cherry_pick, "Apply (cherry-pick)");
    assert_eq!(s.title_mixed_reset_to, "Mixed reset to");
    assert_eq!(s.title_delete_tag, "Delete tag");
    assert_eq!(s.title_create_tag_at, "Create tag at");
    assert_eq!(s.title_rebase_merges, "Rebase merges");
    assert_eq!(
        s.fmt1(s.title_fetch_branch_from_fmt, "origin"),
        "Fetch branch from origin"
    );
}

#[test]
fn test_swedish_select_popup_titles() {
    let s = i18n::swedish();
    assert_eq!(s.title_checkout, "Byt till");
    assert_eq!(
        s.title_create_branch_starting_at,
        "Skapa gren som utgår från"
    );
    assert_eq!(s.title_push_to, "Knuffa till");
    assert_eq!(s.title_push_to_push_remote, "Knuffa till knuffjärr");
    assert_eq!(s.title_pull_from, "Rycka från");
    assert_eq!(s.title_fetch_from, "Hämta från");
    assert_eq!(
        s.title_merge_branch_no_commit,
        "Sammanfoga gren (ingen förbindelse)"
    );
    assert_eq!(s.title_rebase_subset_onto, "Ympa delmängd på");
    assert_eq!(s.title_apply_cherry_pick, "Applicera (plocka russin)");
    assert_eq!(s.title_mixed_reset_to, "Blandat återställ till");
    assert_eq!(s.title_delete_tag, "Fimpa märke");
    assert_eq!(s.title_create_tag_at, "Skapa märke vid");
    assert_eq!(s.title_rebase_merges, "Ympa sammanslagningar");
    assert_eq!(
        s.fmt1(s.title_fetch_branch_from_fmt, "origin"),
        "Hämta gren från origin"
    );
}

#[test]
fn test_english_mode_labels() {
    let s = i18n::english();
    assert_eq!(s.mode_normal, "NORMAL");
    assert_eq!(s.mode_visual, "VISUAL");
    assert_eq!(s.mode_search, "SEARCH");
    assert_eq!(s.mode_preview, "PREVIEW");
}

#[test]
fn test_swedish_mode_labels() {
    let s = i18n::swedish();
    assert_eq!(s.mode_normal, "NORMAL");
    assert_eq!(s.mode_visual, "VISUELL");
    assert_eq!(s.mode_search, "SÖK");
    assert_eq!(s.mode_preview, "FÖRHANDSGRANSKNING");
}

#[test]
fn test_english_fmt1() {
    let s = i18n::english();
    assert_eq!(
        s.fmt1(s.input_rename_branch_fmt, "main"),
        "Rename branch 'main' to:"
    );
    assert_eq!(
        s.fmt1(s.input_worktree_path_fmt, "feature"),
        "Worktree path for 'feature'"
    );
    assert_eq!(
        s.fmt1(s.completed_successfully_fmt, "Rebase"),
        "Rebase completed successfully"
    );
}

#[test]
fn test_swedish_fmt1() {
    let s = i18n::swedish();
    assert_eq!(
        s.fmt1(s.input_rename_branch_fmt, "main"),
        "Byt namn på gren 'main' till:"
    );
    assert_eq!(
        s.fmt1(s.completed_successfully_fmt, "Ombasera"),
        "Ombasera slutfördes"
    );
}

#[test]
fn test_english_operation_names() {
    let s = i18n::english();
    assert_eq!(s.op_apply, "Apply");
    assert_eq!(s.op_merge, "Merge");
    assert_eq!(s.op_push, "Push");
    assert_eq!(s.op_rebase, "Rebase");
    assert_eq!(s.op_revert, "Revert");
    assert_eq!(s.op_stash, "Stash");
    assert_eq!(s.op_stash_keeping_index, "Stash keeping index");
    assert_eq!(s.op_drop_all_stashes, "Drop all stashes");
    // The operation name is interpolated into the success toast.
    assert_eq!(
        s.fmt1(s.completed_successfully_fmt, s.op_merge),
        "Merge completed successfully"
    );
    assert_eq!(
        s.fmt1(s.op_push_to_fmt, "origin/main"),
        "Push to origin/main"
    );
    assert_eq!(s.fmt1(s.op_pull_from_fmt, "origin"), "Pull from origin");
    assert_eq!(s.fmt1(s.op_rebase_onto_fmt, "main"), "Rebase onto main");
    assert_eq!(
        s.fmt2(s.op_push_local_to_fmt, "topic", "origin"),
        "Push topic to origin"
    );
}

#[test]
fn test_swedish_operation_names() {
    let s = i18n::swedish();
    assert_eq!(s.op_apply, "Applicera");
    assert_eq!(s.op_merge, "Sammanfoga");
    assert_eq!(s.op_push, "Knuffa");
    assert_eq!(s.op_rebase, "Ympa");
    assert_eq!(s.op_revert, "Återgå");
    assert_eq!(s.op_stash, "Gömma");
    assert_eq!(s.op_stash_keeping_index, "Gömma med behållet register");
    assert_eq!(s.op_drop_all_stashes, "Fimpa alla gömmor");
    assert_eq!(
        s.fmt1(s.completed_successfully_fmt, s.op_merge),
        "Sammanfoga slutfördes"
    );
    assert_eq!(
        s.fmt1(s.op_push_to_fmt, "origin/main"),
        "Knuffa till origin/main"
    );
    assert_eq!(s.fmt1(s.op_pull_from_fmt, "origin"), "Rycka från origin");
    assert_eq!(s.fmt1(s.op_rebase_onto_fmt, "main"), "Ympa på main");
    assert_eq!(
        s.fmt2(s.op_push_local_to_fmt, "topic", "origin"),
        "Knuffa topic till origin"
    );
}

#[test]
fn test_english_input_titles() {
    let s = i18n::english();
    assert_eq!(s.input_new_branch, "Name for new branch");
    assert_eq!(s.input_spinoff_branch, "Name for new spin-off branch");
    assert_eq!(s.input_spinout_branch, "Name for new spin-out branch");
    assert_eq!(s.input_tag_name, "Tag name");
    assert_eq!(s.input_stash_message, "Stash message");
    assert_eq!(s.input_stash_index_message, "Stash index message");
    assert_eq!(s.input_stash_worktree_message, "Stash worktree message");
}

#[test]
fn test_swedish_input_titles() {
    let s = i18n::swedish();
    assert_eq!(s.input_new_branch, "Namnge ny gren");
    assert_eq!(s.input_tag_name, "Namnge märke");
    assert_eq!(s.input_stash_message, "Namnge gömställe");
}

#[test]
fn test_english_hints() {
    let s = i18n::english();
    assert_eq!(s.hint_dismiss, "Press Enter or Esc to dismiss");
    assert_eq!(s.hint_confirm, "y/Enter to confirm, n/Esc to cancel");
    assert_eq!(s.operation_fallback, "Operation");
}

#[test]
fn test_swedish_hints() {
    let s = i18n::swedish();
    assert_eq!(s.hint_dismiss, "Tryck Enter eller Esc för att stänga");
    assert_eq!(
        s.hint_confirm,
        "y/Enter för att bekräfta, n/Esc för att avbryta"
    );
    assert_eq!(s.operation_fallback, "Åtgärd");
}
