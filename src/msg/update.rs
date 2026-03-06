use crate::{
    model::{InputContext, Model},
    msg::{InputMessage, Message, SearchMessage, SelectMessage},
};

mod amend;
mod checkout_branch;
mod checkout_new_branch;
mod commit;
mod confirm_delete_branch;
mod confirm_discard;
mod confirm_drop_stash;
mod confirm_pop_stash;
mod credentials_input;
mod delete_branch;
mod discard_selected;
mod dismiss_popup;
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
mod input_input;
mod navigation;
mod open_pr;
mod pending_g;
mod pty_helper;
mod pull;
mod push;
mod quit;
mod rebase;
mod refresh;
mod rename_branch;
mod reset_branch;
mod reset_index;
mod reset_worktree;
mod revert;
mod search;
mod select_confirm;
mod select_input_backspace;
mod select_input_char;
mod select_move_down;
mod select_move_up;
mod selection;
mod show_checkout_new_branch_input;
mod show_commit_select;
mod show_fetch_popup;
mod show_input_popup;
mod show_log;
mod show_preview;
mod show_pull_popup;
mod show_push_popup;
mod show_rebase_popup;
mod show_reset_popup;
mod show_revert_popup;
mod show_select_popup;
mod spinoff_branch;
mod spinout_branch;
mod stage_all_modified;
mod stage_selected;
mod stash;
mod toggle_argument;
mod toggle_section;
mod unstage_all;
mod unstage_selected;
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
            SelectMessage::InputChar(c) => select_input_char::update(model, c),
            SelectMessage::InputBackspace => select_input_backspace::update(model),
            SelectMessage::MoveUp => select_move_up::update(model),
            SelectMessage::MoveDown => select_move_down::update(model),
            SelectMessage::Confirm => select_confirm::update(model),
        },
        Message::Input(input_msg) => match input_msg {
            InputMessage::InputChar(c) => input_input::input_char(model, c),
            InputMessage::InputBackspace => input_input::input_backspace(model),
            InputMessage::Confirm => input_input::confirm(model),
        },
        Message::Credentials(credentials_msg) => credentials_input::update(model, credentials_msg),
        Message::ShowLog(log_type) => show_log::update(model, log_type),
        Message::ExitLogView => exit_log_view::update(model),

        Message::EnterSearchMode => enter_search_mode::update(model),
        Message::Search(search_msg) => match search_msg {
            SearchMessage::InputChar(c) => search::input_char(model, c),
            SearchMessage::InputBackspace => search::input_backspace(model),
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
        Message::Rebase(rebase_command) => rebase::update(model, rebase_command),
        Message::ShowRevertPopup => show_revert_popup::update(model),
        Message::Revert(revert_command) => revert::update(model, revert_command),
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
        Message::ShowPreview => show_preview::update(model),
        Message::ExitPreview => exit_preview::update(model),
        Message::FileCheckout { revision, file } => file_checkout::update(model, revision, file),
    }
}
