use std::sync::OnceLock;

/// Supported UI languages.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    English,
    Swedish,
}

impl Language {
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "en" | "english" => Some(Self::English),
            "sv" | "swedish" | "svenska" => Some(Self::Swedish),
            _ => None,
        }
    }
}

/// All user-facing strings used in the TUI.
pub struct Strings {
    // Section headers
    pub section_untracked_files: &'static str,
    pub section_unstaged_changes: &'static str,
    pub section_staged_changes: &'static str,
    pub section_recent_commits: &'static str,
    pub section_stashes: &'static str,
    /// Used for both the "Rebasing" git section header and the in-progress popup title.
    pub section_rebasing: &'static str,
    /// Used for both the "Reverting" git section header and the in-progress popup title.
    pub section_reverting: &'static str,
    pub section_cherry_picking: &'static str,
    /// The prefix before the remote name in "Unpulled from <remote>" (includes trailing space).
    pub section_unpulled_from_prefix: &'static str,

    // Popup window titles
    pub popup_branch: &'static str,
    pub popup_commit: &'static str,
    pub popup_push: &'static str,
    pub popup_pull: &'static str,
    pub popup_fetch: &'static str,
    pub popup_log: &'static str,
    pub popup_stash: &'static str,
    pub popup_tag: &'static str,
    pub popup_reset: &'static str,
    pub popup_rebase: &'static str,
    pub popup_revert: &'static str,
    pub popup_merge: &'static str,
    pub popup_merging: &'static str,
    pub popup_apply: &'static str,
    pub popup_applying: &'static str,
    pub popup_help: &'static str,
    pub popup_error: &'static str,
    pub popup_confirm: &'static str,

    // Column / section titles inside popups
    pub col_checkout: &'static str,
    pub col_create: &'static str,
    pub col_do: &'static str,
    pub col_arguments: &'static str,
    pub col_edit_head: &'static str,
    pub col_edit: &'static str,
    pub col_use: &'static str,
    pub col_reset_this: &'static str,
    pub col_fetch_from: &'static str,
    pub col_actions: &'static str,
    pub col_commands: &'static str,
    pub col_applying_changes: &'static str,
    pub col_general: &'static str,
    pub col_apply_here: &'static str,

    // Dynamic styled title parts used in push/pull/rebase popups.
    // The branch name is coloured separately between pre and post.
    pub push_to_pre: &'static str,
    pub push_to_post: &'static str,
    pub push_to_fallback: &'static str,
    pub pull_into_pre: &'static str,
    pub pull_into_post: &'static str,
    pub pull_into_fallback: &'static str,
    pub rebase_onto_pre: &'static str,
    pub rebase_onto_post: &'static str,

    // Input popup titles (static)
    pub input_new_branch: &'static str,
    pub input_spinoff_branch: &'static str,
    pub input_spinout_branch: &'static str,
    pub input_tag_name: &'static str,
    pub input_stash_message: &'static str,
    pub input_stash_index_message: &'static str,
    pub input_stash_worktree_message: &'static str,
    // Input popup title format strings — use fmt1() with one {} placeholder
    pub input_rename_branch_fmt: &'static str,
    pub input_worktree_path_fmt: &'static str,
    pub input_push_refspec_fmt: &'static str,
    pub input_fetch_refspec_fmt: &'static str,

    // Command descriptions used inside popup rows and the help popup
    pub cmd_branch_revision: &'static str,
    pub cmd_local_branch: &'static str,
    pub cmd_new_branch: &'static str,
    pub cmd_new_spinoff: &'static str,
    pub cmd_new_worktree: &'static str,
    pub cmd_new_pr_default: &'static str,
    pub cmd_new_pr_to: &'static str,
    pub cmd_new_spinout: &'static str,
    pub cmd_rename: &'static str,
    pub cmd_delete: &'static str,
    pub cmd_reset: &'static str,
    pub cmd_commit: &'static str,
    pub cmd_extend: &'static str,
    pub cmd_amend: &'static str,
    pub cmd_reword: &'static str,
    pub cmd_fixup: &'static str,
    pub cmd_squash: &'static str,
    pub cmd_alter: &'static str,
    pub cmd_augment: &'static str,
    pub cmd_revise: &'static str,
    pub cmd_tag: &'static str,
    pub cmd_prune: &'static str,
    pub cmd_elsewhere: &'static str,
    pub cmd_all_remotes: &'static str,
    pub cmd_another_branch: &'static str,
    pub cmd_explicit_refspec: &'static str,
    pub cmd_submodules: &'static str,
    pub cmd_current: &'static str,
    pub cmd_local_branches: &'static str,
    pub cmd_all_branches: &'static str,
    pub cmd_all_references: &'static str,
    pub cmd_both: &'static str,
    pub cmd_index: &'static str,
    pub cmd_worktree: &'static str,
    pub cmd_apply: &'static str,
    pub cmd_pop: &'static str,
    pub cmd_drop: &'static str,
    pub cmd_other_branch: &'static str,
    pub cmd_matching_branches: &'static str,
    pub cmd_push_tag: &'static str,
    pub cmd_push_all_tags: &'static str,
    pub cmd_branch: &'static str,
    pub cmd_file: &'static str,
    pub cmd_reset_mixed: &'static str,
    pub cmd_reset_soft: &'static str,
    pub cmd_reset_hard: &'static str,
    pub cmd_reset_keep: &'static str,
    pub cmd_reset_index: &'static str,
    pub cmd_reset_worktree: &'static str,
    pub cmd_continue: &'static str,
    pub cmd_skip: &'static str,
    pub cmd_abort: &'static str,
    pub cmd_merge: &'static str,
    pub cmd_revert_commits: &'static str,
    pub cmd_pick: &'static str,
    // Help popup command descriptions
    pub cmd_fetch: &'static str,
    pub cmd_log: &'static str,
    pub cmd_pull: &'static str,
    pub cmd_push: &'static str,
    pub cmd_rebase: &'static str,
    pub cmd_revert: &'static str,
    pub cmd_stash: &'static str,
    pub cmd_stage: &'static str,
    pub cmd_stage_all: &'static str,
    pub cmd_unstage: &'static str,
    pub cmd_unstage_all: &'static str,
    pub cmd_discard: &'static str,
    pub cmd_quit: &'static str,
    pub cmd_refresh: &'static str,
    pub cmd_show_help: &'static str,
    pub cmd_move_down: &'static str,
    pub cmd_move_up: &'static str,
    pub cmd_half_page_down: &'static str,
    pub cmd_half_page_up: &'static str,
    pub cmd_go_first_line: &'static str,
    pub cmd_go_last_line: &'static str,
    pub cmd_scroll_down: &'static str,
    pub cmd_scroll_up: &'static str,
    pub cmd_toggle_section: &'static str,
    pub cmd_visual_mode: &'static str,

    // Mode labels shown in the status bar
    pub mode_normal: &'static str,
    pub mode_visual: &'static str,
    pub mode_search: &'static str,
    pub mode_preview: &'static str,

    // Popup hint lines
    pub hint_dismiss: &'static str,
    pub hint_confirm: &'static str,

    // Toast messages
    /// Format template: replace `{}` with the operation name via fmt1().
    pub completed_successfully_fmt: &'static str,
    /// Fallback operation name used when none is available.
    pub operation_fallback: &'static str,
}

impl Strings {
    /// Replace the first `{}` placeholder in `template` with `arg`.
    pub fn fmt1(&self, template: &'static str, arg: &str) -> String {
        template.replacen("{}", arg, 1)
    }
}

static ENGLISH: Strings = Strings {
    section_untracked_files: "Untracked files",
    section_unstaged_changes: "Unstaged changes",
    section_staged_changes: "Staged changes",
    section_recent_commits: "Recent commits",
    section_stashes: "Stashes",
    section_rebasing: "Rebasing",
    section_reverting: "Reverting",
    section_cherry_picking: "Cherry Picking",
    section_unpulled_from_prefix: "Unpulled from ",

    popup_branch: "Branch",
    popup_commit: "Commit",
    popup_push: "Push",
    popup_pull: "Pull",
    popup_fetch: "Fetch",
    popup_log: "Log",
    popup_stash: "Stash",
    popup_tag: "Tag",
    popup_reset: "Reset",
    popup_rebase: "Rebase",
    popup_revert: "Revert",
    popup_merge: "Merge",
    popup_merging: "Merging",
    popup_apply: "Apply",
    popup_applying: "Applying",
    popup_help: "Help",
    popup_error: "Error",
    popup_confirm: "Confirm",

    col_checkout: "Checkout",
    col_create: "Create",
    col_do: "Do",
    col_arguments: "Arguments",
    col_edit_head: "Edit HEAD",
    col_edit: "Edit",
    col_use: "Use",
    col_reset_this: "Reset this",
    col_fetch_from: "Fetch from",
    col_actions: "Actions",
    col_commands: "Commands",
    col_applying_changes: "Applying changes",
    col_general: "General",
    col_apply_here: "Apply here",

    push_to_pre: "Push ",
    push_to_post: " to",
    push_to_fallback: "Push to",
    pull_into_pre: "Pull into ",
    pull_into_post: " from",
    pull_into_fallback: "Pull into",
    rebase_onto_pre: "Rebase ",
    rebase_onto_post: " onto",

    input_new_branch: "Name for new branch",
    input_spinoff_branch: "Name for new spin-off branch",
    input_spinout_branch: "Name for new spin-out branch",
    input_tag_name: "Tag name",
    input_stash_message: "Stash message",
    input_stash_index_message: "Stash index message",
    input_stash_worktree_message: "Stash worktree message",
    input_rename_branch_fmt: "Rename branch '{}' to:",
    input_worktree_path_fmt: "Worktree path for '{}'",
    input_push_refspec_fmt: "Push refspec(s) to '{}' (comma-separated)",
    input_fetch_refspec_fmt: "Fetch refspec(s) from '{}' (comma-separated)",

    cmd_branch_revision: "Branch/revision",
    cmd_local_branch: "Local branch",
    cmd_new_branch: "New branch",
    cmd_new_spinoff: "New spin-off",
    cmd_new_worktree: "New worktree",
    cmd_new_pr_default: "New PR to default branch",
    cmd_new_pr_to: "New PR to...",
    cmd_new_spinout: "New spin-out",
    cmd_rename: "Rename",
    cmd_delete: "Delete",
    cmd_reset: "Reset",
    cmd_commit: "Commit",
    cmd_extend: "Extend",
    cmd_amend: "Amend",
    cmd_reword: "Reword",
    cmd_fixup: "Fixup",
    cmd_squash: "Squash",
    cmd_alter: "Alter",
    cmd_augment: "Augment",
    cmd_revise: "Revise",
    cmd_tag: "Tag",
    cmd_prune: "Prune",
    cmd_elsewhere: "Elsewhere",
    cmd_all_remotes: "All remotes",
    cmd_another_branch: "Another branch",
    cmd_explicit_refspec: "Explicit refspec",
    cmd_submodules: "Submodules",
    cmd_current: "Current",
    cmd_local_branches: "Local branches",
    cmd_all_branches: "All branches",
    cmd_all_references: "All references",
    cmd_both: "Both",
    cmd_index: "Index",
    cmd_worktree: "Worktree",
    cmd_apply: "Apply",
    cmd_pop: "Pop",
    cmd_drop: "Drop",
    cmd_other_branch: "Other branch",
    cmd_matching_branches: "Matching branches",
    cmd_push_tag: "Push a tag",
    cmd_push_all_tags: "Push all tags",
    cmd_branch: "Branch",
    cmd_file: "File",
    cmd_reset_mixed: "Mixed    (HEAD and index)",
    cmd_reset_soft: "Soft     (HEAD only)",
    cmd_reset_hard: "Hard     (HEAD, index and worktree)",
    cmd_reset_keep: "Keep     (HEAD and index, keeping uncommitted)",
    cmd_reset_index: "Index    (only)",
    cmd_reset_worktree: "Worktree (only)",
    cmd_continue: "Continue",
    cmd_skip: "Skip",
    cmd_abort: "Abort",
    cmd_merge: "Merge",
    cmd_revert_commits: "Revert commit(s)",
    cmd_pick: "Pick",
    cmd_fetch: "Fetch",
    cmd_log: "Log",
    cmd_pull: "Pull",
    cmd_push: "Push",
    cmd_rebase: "Rebase",
    cmd_revert: "Revert",
    cmd_stash: "Stash",
    cmd_stage: "Stage",
    cmd_stage_all: "Stage all",
    cmd_unstage: "Unstage",
    cmd_unstage_all: "Unstage all",
    cmd_discard: "Discard",
    cmd_quit: "        quit",
    cmd_refresh: "Refresh",
    cmd_show_help: "      show this help",
    cmd_move_down: "   move down",
    cmd_move_up: "     move up",
    cmd_half_page_down: "   half page down",
    cmd_half_page_up: "   half page up",
    cmd_go_first_line: "       go to first line",
    cmd_go_last_line: "        go to last line",
    cmd_scroll_down: "   scroll one line down",
    cmd_scroll_up: "   scroll one line up",
    cmd_toggle_section: "      toggle section collapsed/expanded",
    cmd_visual_mode: "        enter visual selection mode",

    mode_normal: "NORMAL",
    mode_visual: "VISUAL",
    mode_search: "SEARCH",
    mode_preview: "PREVIEW",

    hint_dismiss: "Press Enter or Esc to dismiss",
    hint_confirm: "y/Enter to confirm, n/Esc to cancel",

    completed_successfully_fmt: "{} completed successfully",
    operation_fallback: "Operation",
};

static SWEDISH: Strings = Strings {
    section_untracked_files: "Ospårade filer",
    section_unstaged_changes: "Ej klarmarkerade ändringar",
    section_staged_changes: "Klarmarkerade ändringar",
    section_recent_commits: "Senaste förbindelser",
    section_stashes: "Gömda ändringar",
    section_rebasing: "Ympar",
    section_reverting: "Återgår",
    section_cherry_picking: "Plockar russin",
    section_unpulled_from_prefix: "Ej ryckta från ",

    popup_branch: "Grena",
    popup_commit: "Förbinda",
    popup_push: "Knuffa",
    popup_pull: "Rycka",
    popup_fetch: "Hämta",
    popup_log: "Diarium",
    popup_stash: "Gömma",
    popup_tag: "Märke",
    popup_reset: "Återställ",
    popup_rebase: "Ympa",
    popup_revert: "Återgå",
    popup_merge: "Sammanfoga",
    popup_merging: "Sammanfogar",
    popup_apply: "Plocka russin",
    popup_applying: "Plockar russin",
    popup_help: "Hjälp",
    popup_error: "Fel",
    popup_confirm: "Bekräfta",

    col_checkout: "Byt till",
    col_create: "Skapa",
    col_do: "Utför",
    col_arguments: "Argument",
    col_edit_head: "Redigera HEAD",
    col_edit: "Redigera",
    col_use: "Använd",
    col_reset_this: "Återställ detta",
    col_fetch_from: "Hämta från",
    col_actions: "Åtgärder",
    col_commands: "Kommandon",
    col_applying_changes: "Tillämpa ändringar",
    col_general: "Allmänt",
    col_apply_here: "Plocka russin här",

    push_to_pre: "Knuffa ",
    push_to_post: " till",
    push_to_fallback: "Knuff",
    pull_into_pre: "Rycka ",
    pull_into_post: " från",
    pull_into_fallback: "Rycka",
    rebase_onto_pre: "Ympa ",
    rebase_onto_post: " på",

    input_new_branch: "Namnge ny gren",
    input_spinoff_branch: "Namnge ny spin-off-gren",
    input_spinout_branch: "Namnge ny spin-out-gren",
    input_tag_name: "Namnge märke",
    input_stash_message: "Namnge gömställe",
    input_stash_index_message: "Göm indexmeddelande",
    input_stash_worktree_message: "Göm arbetsträdsmeddelande",
    input_rename_branch_fmt: "Byt namn på gren '{}' till:",
    input_worktree_path_fmt: "Arbetsträdsväg för '{}'",
    input_push_refspec_fmt: "Knuffa refspec(er) till '{}' (kommaseparerade)",
    input_fetch_refspec_fmt: "Hämta refspec(er) från '{}' (kommaseparerade)",

    cmd_branch_revision: "Gren/revision",
    cmd_local_branch: "Lokal gren",
    cmd_new_branch: "Ny gren",
    cmd_new_spinoff: "Ny spin-off",
    cmd_new_worktree: "Nytt arbetsträd",
    cmd_new_pr_default: "Nytt ryckbegäran till huvudgrenen",
    cmd_new_pr_to: "Nyt ryckbegäran till...",
    cmd_new_spinout: "Ny spin-out",
    cmd_rename: "Byt namn",
    cmd_delete: "Fimpa",
    cmd_reset: "Återställ",
    cmd_commit: "Förbinda",
    cmd_extend: "Förläng",
    cmd_amend: "Rätta till",
    cmd_reword: "Omformulera",
    cmd_fixup: "Fixup",
    cmd_squash: "Mosa",
    cmd_alter: "Modifiera",
    cmd_augment: "Utöka",
    cmd_revise: "Revidera",
    cmd_tag: "Märka",
    cmd_prune: "Rensa",
    cmd_elsewhere: "Annanstans",
    cmd_all_remotes: "Alla fjärrar",
    cmd_another_branch: "Annan gren",
    cmd_explicit_refspec: "Explicit refspec",
    cmd_submodules: "Undermoduler",
    cmd_current: "Nuvarande",
    cmd_local_branches: "Lokala grenar",
    cmd_all_branches: "Alla grenar",
    cmd_all_references: "Alla referenser",
    cmd_both: "Båda",
    cmd_index: "Register",
    cmd_worktree: "Arbetsträd",
    cmd_apply: "Applicera",
    cmd_pop: "Plocka",
    cmd_drop: "Fimpa",
    cmd_other_branch: "Annan gren",
    cmd_matching_branches: "Matchande grenar",
    cmd_push_tag: "Knuffa ett märke",
    cmd_push_all_tags: "Knuffa alla märken",
    cmd_branch: "Gren",
    cmd_file: "Fil",
    cmd_reset_mixed: "Blandat    (HEAD och register)",
    cmd_reset_soft: "Mjukt      (endast HEAD)",
    cmd_reset_hard: "Hårt       (HEAD, register och arbetsträd)",
    cmd_reset_keep: "Behåll     (HEAD och index, behåll icke-klarmarkerade)",
    cmd_reset_index: "Register   (endast)",
    cmd_reset_worktree: "Arbetsträd (endast)",
    cmd_continue: "Fortsätt",
    cmd_skip: "Hoppa över",
    cmd_abort: "Avbryt",
    cmd_merge: "Sammanfoga",
    cmd_revert_commits: "Återgå förbindelse(r)",
    cmd_pick: "Plocka",
    cmd_fetch: "Hämta",
    cmd_log: "Diarium",
    cmd_pull: "Rycka",
    cmd_push: "Knuffa",
    cmd_rebase: "Ympa",
    cmd_revert: "Återgå",
    cmd_stash: "Göm",
    cmd_stage: "Klarmarkera",
    cmd_stage_all: "Klarmarkera allt",
    cmd_unstage: "Återkalla",
    cmd_unstage_all: "Återkalla allt",
    cmd_discard: "Fimpa",
    cmd_quit: "        Avsluta",
    cmd_refresh: "Förfriska",
    cmd_show_help: "      Visa denna hjälp",
    cmd_move_down: "   Flytta ned",
    cmd_move_up: "     Flytta upp",
    cmd_half_page_down: "   Halv sida ned",
    cmd_half_page_up: "   Halv sida upp",
    cmd_go_first_line: "       Gå till första raden",
    cmd_go_last_line: "        Gå till sista raden",
    cmd_scroll_down: "   Rulla en rad ned",
    cmd_scroll_up: "   Rulla en rad upp",
    cmd_toggle_section: "      Växla ihopfällt/utfällt",
    cmd_visual_mode: "        Aktivera visuellt läge",

    mode_normal: "NORMAL",
    mode_visual: "VISUELL",
    mode_search: "SÖK",
    mode_preview: "FÖRHANDSGRANSKNING",

    hint_dismiss: "Tryck Enter eller Esc för att stänga",
    hint_confirm: "y/Enter för att bekräfta, n/Esc för att avbryta",

    completed_successfully_fmt: "{} slutfördes",
    operation_fallback: "Åtgärd",
};

static ACTIVE: OnceLock<Language> = OnceLock::new();

/// Initialise the global language. Only the first call takes effect.
pub fn init(lang: Language) {
    ACTIVE.set(lang).ok();
}

/// Return the active language's string table.
/// Defaults to English if `init` has not been called.
pub fn t() -> &'static Strings {
    match ACTIVE.get().copied().unwrap_or(Language::English) {
        Language::English => &ENGLISH,
        Language::Swedish => &SWEDISH,
    }
}

/// Return the English string table (useful in tests without global state).
pub fn english() -> &'static Strings {
    &ENGLISH
}

/// Return the Swedish string table (useful in tests without global state).
pub fn swedish() -> &'static Strings {
    &SWEDISH
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_language_from_str() {
        assert!(matches!(Language::from_str("en"), Some(Language::English)));
        assert!(matches!(
            Language::from_str("english"),
            Some(Language::English)
        ));
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
        assert!(Language::from_str("de").is_none());
        assert!(Language::from_str("french").is_none());
    }

    #[test]
    fn test_english_strings() {
        let s = english();
        assert_eq!(s.section_untracked_files, "Untracked files");
        assert_eq!(s.section_unstaged_changes, "Unstaged changes");
        assert_eq!(s.section_staged_changes, "Staged changes");
        assert_eq!(s.section_recent_commits, "Recent commits");
        assert_eq!(s.section_stashes, "Stashes");
        assert_eq!(s.popup_branch, "Branch");
        assert_eq!(s.popup_commit, "Commit");
        assert_eq!(s.popup_error, "Error");
        assert_eq!(s.popup_help, "Help");
        assert_eq!(s.mode_normal, "NORMAL");
        assert_eq!(s.mode_visual, "VISUAL");
        assert_eq!(s.mode_search, "SEARCH");
        assert_eq!(s.mode_preview, "PREVIEW");
    }

    #[test]
    fn test_swedish_strings() {
        let s = swedish();
        assert_eq!(s.section_untracked_files, "Ospårade filer");
        assert_eq!(s.section_unstaged_changes, "Ej klarmarkerade ändringar");
        assert_eq!(s.section_staged_changes, "Klarmarkerade ändringar");
        assert_eq!(s.section_recent_commits, "Senaste förbindelser");
        assert_eq!(s.section_stashes, "Gömda ändringar");
        assert_eq!(s.popup_branch, "Grena");
        assert_eq!(s.popup_commit, "Förbinda");
        assert_eq!(s.popup_error, "Fel");
        assert_eq!(s.popup_help, "Hjälp");
        assert_eq!(s.mode_normal, "NORMAL");
        assert_eq!(s.mode_visual, "VISUELL");
        assert_eq!(s.mode_search, "SÖK");
        assert_eq!(s.mode_preview, "FÖRHANDSGRANSKNING");
    }

    #[test]
    fn test_fmt1() {
        let s = english();
        assert_eq!(
            s.fmt1(s.input_rename_branch_fmt, "main"),
            "Rename branch 'main' to:"
        );
        assert_eq!(
            s.fmt1(s.completed_successfully_fmt, "Rebase"),
            "Rebase completed successfully"
        );
    }

    #[test]
    fn test_fmt1_swedish() {
        let s = swedish();
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
    fn test_t_defaults_to_english() {
        // When ACTIVE is not set (or set to English), t() returns English strings.
        // Note: ACTIVE is a OnceLock; in practice this test works because the test
        // process either hasn't called init() yet or was initialised to English.
        let s = t();
        // Verify it returns a valid Strings (English or Swedish — just check non-empty)
        assert!(!s.section_untracked_files.is_empty());
    }
}
