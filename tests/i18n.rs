use magi::i18n::{self, Language};

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
    assert_eq!(s.section_unstaged_changes, "Ej iscensatta ändringar");
    assert_eq!(s.section_staged_changes, "Iscensatta ändringar");
    assert_eq!(s.section_recent_commits, "Senaste incheckningar");
    assert_eq!(s.section_stashes, "Tillfälliga ändringar");
    assert_eq!(s.section_rebasing, "Ombaserar");
    assert_eq!(s.section_reverting, "Återgår");
    assert_eq!(s.section_cherry_picking, "Plockar");
    assert_eq!(s.section_unpulled_from_prefix, "Ej hämtade från ");
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
    assert_eq!(s.popup_branch, "Gren");
    assert_eq!(s.popup_commit, "Incheckning");
    assert_eq!(s.popup_push, "Skicka");
    assert_eq!(s.popup_pull, "Hämta");
    assert_eq!(s.popup_fetch, "Hämta ned");
    assert_eq!(s.popup_merge, "Sammanfoga");
    assert_eq!(s.popup_merging, "Sammanfogar");
    assert_eq!(s.popup_error, "Fel");
    assert_eq!(s.popup_help, "Hjälp");
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
    assert_eq!(s.input_new_branch, "Namn för ny gren");
    assert_eq!(s.input_tag_name, "Taggnamn");
    assert_eq!(s.input_stash_message, "Göm meddelande");
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
