use crate::{
    model::{InputContext, Model},
    msg::{InputMessage, Message, SearchMessage, SelectMessage},
};

mod amend;
mod apply;
mod apply_selected;
mod checkout_branch;
mod checkout_new_branch;
mod cherry_spinoff;
mod cherry_spinout;
mod commit;
mod confirm_delete_branch;
mod confirm_discard;
mod confirm_drop_stash;
mod confirm_pop_stash;
mod confirm_reverse;
mod create_tag;
mod create_tag_release;
mod credentials_input;
mod delete_branch;
mod delete_tag;
mod discard_selected;
mod dismiss_popup;
mod donate;
mod enter_arg_mode;
mod enter_search_mode;
mod enter_visual_mode;
mod exit_arg_mode;
mod exit_log_view;
mod exit_preview;
mod exit_visual_mode;
mod fetch;
mod file_checkout;
mod fixup_commit;
mod harvest;
mod input_input;
mod merge;
mod navigation;
mod open_pr;
mod pending_g;
mod prune_tags;
mod pty_helper;
mod pull;
mod push;
mod quit;
mod rebase;
mod rebase_todo;
mod refresh;
mod rename_branch;
mod reset_branch;
mod reset_index;
mod reset_worktree;
mod reverse_selected;
mod revert;
mod revise_commit;
mod search;
mod select_confirm;
mod select_edit;
mod select_move_down;
mod select_move_up;
mod selection;
mod show_apply_popup;
mod show_checkout_new_branch_input;
mod show_commit_author_select;
mod show_commit_select;
mod show_fetch_popup;
mod show_input_popup;
mod show_log;
mod show_log_popup;
mod show_merge_popup;
mod show_preview;
mod show_prune_tags_confirm;
mod show_pull_popup;
mod show_push_popup;
mod show_rebase_popup;
mod show_reset_popup;
mod show_revert_mainline_input;
mod show_revert_popup;
mod show_select_popup;
mod show_tag_popup;
mod show_tag_release_input;
mod spinoff_branch;
mod spinout_branch;
mod stage_all_modified;
mod stage_selected;
mod stash;
mod toggle_argument;
mod toggle_section;
mod unstage_all;
mod unstage_selected;
mod worktree_branch;
mod worktree_checkout;

/// Processes a [`Message`], modifying the passed model.
///
/// Returns a follow up [`Message`] for sequences of actions.
/// e.g. after a stage, a [`Message::Refresh`] should be triggered.
pub fn update(model: &mut Model, msg: Message) -> Option<Message> {
    // Clear pending 'g' state for any message except PendingG itself
    if !matches!(msg, Message::PendingG) {
        model.pending_g = false;
    }

    match msg {
        Message::Quit => quit::update(model),
        Message::Refresh => refresh::update(model),
        Message::ToggleSection => toggle_section::update(model),
        Message::Navigation(action) => navigation::update(model, action),
        Message::PendingG => pending_g::update(model),
        Message::Commit => commit::update(model),
        Message::ShowCommitAuthorSelect => show_commit_author_select::update(model),
        Message::Amend(extra_args) => amend::update(model, extra_args),
        Message::FixupCommit(commit_hash, fixup_type) => {
            fixup_commit::update(model, commit_hash, fixup_type)
        }
        Message::DismissPopup => dismiss_popup::update(model),
        Message::ShowStashInput(stash_type) => {
            show_input_popup::update(model, InputContext::Stash(stash_type))
        }
        Message::StageAllModified => stage_all_modified::update(model),
        Message::StageSelected => stage_selected::update(model),
        Message::UnstageSelected => unstage_selected::update(model),
        Message::UnstageAll => unstage_all::update(model),
        Message::ApplySelected => apply_selected::update(model),
        Message::ReverseSelected => reverse_selected::update(model),
        Message::ConfirmReverse(target) => confirm_reverse::update(model, target),
        Message::DiscardSelected => discard_selected::update(model),
        Message::ConfirmDiscard(target) => confirm_discard::update(model, target),
        Message::ConfirmPopStash(stash_ref) => confirm_pop_stash::update(model, stash_ref),
        Message::ConfirmDropStash(stash_ref) => confirm_drop_stash::update(model, stash_ref),
        Message::EnterVisualMode => enter_visual_mode::update(model),
        Message::ExitVisualMode => exit_visual_mode::update(model),
        Message::ShowPopup(content) => {
            model.popup = Some(content);
            None
        }
        Message::ShowPushPopup => show_push_popup::update(model),
        Message::ShowFetchPopup => show_fetch_popup::update(model),
        Message::ShowPullPopup => show_pull_popup::update(model),
        Message::ShowRenameBranchInput(old_name) => {
            show_input_popup::update(model, InputContext::RenameBranch { old_name })
        }
        Message::ShowPushRefspecInput(remote) => {
            show_input_popup::update(model, InputContext::PushRefspec { remote })
        }
        Message::ShowFetchRefspecInput(remote) => {
            show_input_popup::update(model, InputContext::FetchRefspec { remote })
        }
        Message::RenameBranch { old_name, new_name } => {
            rename_branch::update(model, old_name, new_name)
        }
        Message::ShowCreateNewBranchInput {
            starting_point,
            checkout,
        } => show_checkout_new_branch_input::update(model, starting_point, checkout),
        Message::CreateNewBranch {
            starting_point,
            branch_name,
            checkout,
        } => checkout_new_branch::update(model, starting_point, branch_name, checkout),
        Message::CheckoutBranch(branch) => checkout_branch::update(model, branch),
        Message::DeleteBranch(branch) => delete_branch::update(model, branch),
        Message::ConfirmDeleteBranch(branch) => confirm_delete_branch::update(model, branch),
        Message::OpenPr { branch, target } => open_pr::update(model, branch, target),
        Message::ShowSelectPopup(popup) => show_select_popup::update(model, popup),
        Message::ShowCommitSelect(commit_select) => {
            show_commit_select::update(model, commit_select)
        }
        Message::EnterArgMode => enter_arg_mode::update(model),
        Message::ExitArgMode => exit_arg_mode::update(model),
        Message::ToggleArgument(argument) => toggle_argument::update(model, argument),
        Message::Select(select_msg) => match select_msg {
            SelectMessage::Edit(op) => select_edit::update(model, op),
            SelectMessage::MoveUp => select_move_up::update(model),
            SelectMessage::MoveDown => select_move_down::update(model),
            SelectMessage::Confirm => select_confirm::update(model),
        },
        Message::Input(input_msg) => match input_msg {
            InputMessage::Edit(op) => input_input::edit(model, op),
            InputMessage::Confirm => input_input::confirm(model),
        },
        Message::Credentials(credentials_msg) => credentials_input::update(model, credentials_msg),
        Message::ShowLogPopup => show_log_popup::update(model),
        Message::ShowLog(log_type) => show_log::update(model, log_type),
        Message::ExitLogView => exit_log_view::update(model),

        Message::EnterSearchMode => enter_search_mode::update(model),
        Message::Search(search_msg) => match search_msg {
            SearchMessage::Edit(op) => search::edit(model, op),
            SearchMessage::Confirm => search::confirm(model),
            SearchMessage::Next => search::next(model),
            SearchMessage::Prev => search::prev(model),
            SearchMessage::Cancel => search::cancel(model),
        },

        Message::Fetch(fetch_command) => fetch::update(model, fetch_command),
        Message::Pull(pull_command) => pull::update(model, pull_command),
        Message::Stash(stash_command) => stash::update(model, stash_command),
        Message::Push(push_command) => push::update(model, push_command),
        Message::ShowRebasePopup => show_rebase_popup::update(model),
        Message::ShowRebaseTodo(base) => rebase_todo::show(model, base),
        Message::RebaseTodo(msg) => rebase_todo::update(model, msg),
        Message::Rebase(rebase_command) => rebase::update(model, rebase_command),
        Message::ShowRevertPopup => show_revert_popup::update(model),
        Message::ShowRevertMainlineInput => show_revert_mainline_input::update(model),
        Message::Revert(revert_command) => revert::update(model, revert_command),
        Message::ShowApplyPopup => show_apply_popup::update(model),
        Message::Apply(apply_command) => apply::update(model, apply_command),
        Message::Harvest { commits, source } => harvest::update(model, commits, source),
        Message::Donate { commits, target } => donate::update(model, commits, target),
        Message::ShowCherrySpinoutInput { commits, root } => {
            show_input_popup::update(model, InputContext::CherrySpinout { commits, root })
        }
        Message::CherrySpinout {
            commits,
            branch,
            root,
        } => cherry_spinout::update(model, commits, branch, root),
        Message::ShowCherrySpinoffInput { commits, root } => {
            show_input_popup::update(model, InputContext::CherrySpinoff { commits, root })
        }
        Message::CherrySpinoff {
            commits,
            branch,
            root,
        } => cherry_spinoff::update(model, commits, branch, root),
        Message::ShowMergePopup => show_merge_popup::update(model),
        Message::ShowTagPopup => show_tag_popup::update(model),
        Message::ShowCreateTagInput => show_input_popup::update(model, InputContext::CreateTag),
        Message::ShowTagReleaseInput => show_tag_release_input::update(model),
        Message::CreateTagRelease { name } => create_tag_release::update(model, name),
        Message::CreateTag { name, target } => create_tag::update(model, name, target),
        Message::CreateTagWithEditor { name, args } => create_tag::with_editor(model, name, args),
        Message::DeleteTag(name) => delete_tag::update(model, name),
        Message::ShowPruneTagsConfirm { remote } => show_prune_tags_confirm::update(model, remote),
        Message::PruneTags {
            local_tags,
            remote_tags,
            remote,
        } => prune_tags::update(model, local_tags, remote_tags, remote),
        Message::Merge(merge_command) => merge::update(model, merge_command),
        Message::ShowResetPopup => show_reset_popup::update(model),
        Message::ResetBranch {
            branch,
            target,
            mode,
        } => reset_branch::update(model, branch, target, mode),
        Message::ResetIndex { target } => reset_index::update(model, target),
        Message::ResetWorktree { target } => reset_worktree::update(model, target),
        Message::ShowSpinoffBranchInput => {
            show_input_popup::update(model, InputContext::SpinoffBranch)
        }
        Message::SpinoffBranch(branch_name) => spinoff_branch::update(model, branch_name),
        Message::ShowSpinoutBranchInput => {
            show_input_popup::update(model, InputContext::SpinoutBranch)
        }
        Message::SpinoutBranch(branch_name) => spinout_branch::update(model, branch_name),
        Message::ShowWorktreePathInput { branch, checkout } => {
            show_input_popup::update(model, InputContext::WorktreePath { branch, checkout })
        }
        Message::WorktreeCheckout {
            branch,
            path,
            checkout,
        } => worktree_checkout::update(model, branch, path, checkout),
        Message::ShowWorktreeBranchNameInput { starting_point } => {
            show_input_popup::update(model, InputContext::WorktreeBranchName { starting_point })
        }
        Message::ShowWorktreeBranchPathInput {
            starting_point,
            branch_name,
        } => show_input_popup::update(
            model,
            InputContext::WorktreeBranchPath {
                starting_point,
                branch_name,
            },
        ),
        Message::WorktreeBranch {
            starting_point,
            branch_name,
            path,
        } => worktree_branch::update(model, starting_point, branch_name, path),
        Message::ShowPreview => show_preview::update(model),
        Message::ExitPreview => exit_preview::update(model),
        Message::FileCheckout { revision, file } => file_checkout::update(model, revision, file),
        Message::ReviseCommit(hash) => revise_commit::update(model, hash),
    }
}
