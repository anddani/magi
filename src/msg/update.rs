use crate::{
    model::Model,
    msg::{InputMessage, Message, SelectDialog, SelectMessage},
};

mod amend;
mod checkout_branch;
mod checkout_new_branch;
mod commit;
mod confirm_delete_branch;
mod confirm_discard;
mod confirm_drop_stash;
mod credentials_input;
mod delete_branch;
mod discard_selected;
mod dismiss_popup;
mod enter_arg_mode;
mod enter_visual_mode;
mod exit_arg_mode;
mod exit_log_view;
mod exit_visual_mode;
mod fetch;
mod fixup_commit;
mod half_page_down;
mod half_page_up;
mod input_input;
mod move_down;
mod move_to_bottom;
mod move_to_top;
mod move_up;
mod open_pr;
mod pending_g;
mod pty_helper;
mod pull;
mod push;
mod quit;
mod refresh;
mod rename_branch;
mod scroll_line_down;
mod scroll_line_up;
mod select_confirm;
mod select_input_backspace;
mod select_input_char;
mod select_move_down;
mod select_move_up;
mod selection;
mod show_checkout_branch_popup;
mod show_checkout_local_branch_popup;
mod show_checkout_new_branch_input;
mod show_checkout_new_branch_popup;
mod show_delete_branch_popup;
mod show_fetch_another_branch_branch_select;
mod show_fetch_another_branch_select;
mod show_fetch_elsewhere_select;
mod show_fetch_popup;
mod show_fetch_push_remote_select;
mod show_fetch_upstream_select;
mod show_fixup_commit_select;
mod show_log;
mod show_open_pr_select;
mod show_open_pr_target_select;
mod show_pull_popup;
mod show_pull_push_remote_select;
mod show_pull_upstream_select;
mod show_push_all_tags_select;
mod show_push_popup;
mod show_push_push_remote_select;
mod show_push_tag_select;
mod show_push_upstream_select;
mod show_rename_branch_input;
mod show_rename_branch_popup;
mod show_select_popup;
mod show_stash_apply_select;
mod show_stash_drop_select;
mod show_stash_message_input;
mod stage_all_modified;
mod stage_selected;
mod stash;
mod toggle_argument;
mod toggle_section;
mod unstage_all;
mod unstage_selected;

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
        Message::MoveUp => move_up::update(model),
        Message::MoveDown => move_down::update(model),
        Message::ToggleSection => toggle_section::update(model),
        Message::HalfPageUp => half_page_up::update(model),
        Message::HalfPageDown => half_page_down::update(model),
        Message::ScrollLineDown => scroll_line_down::update(model),
        Message::ScrollLineUp => scroll_line_up::update(model),
        Message::MoveToTop => move_to_top::update(model),
        Message::MoveToBottom => move_to_bottom::update(model),
        Message::PendingG => pending_g::update(model),
        Message::Commit => commit::update(model),
        Message::Amend(extra_args) => amend::update(model, extra_args),
        Message::FixupCommit(commit_hash, fixup_type) => {
            fixup_commit::update(model, commit_hash, fixup_type)
        }
        Message::DismissPopup => dismiss_popup::update(model),
        Message::ShowStashMessageInput => show_stash_message_input::update(model),
        Message::StageAllModified => stage_all_modified::update(model),
        Message::StageSelected => stage_selected::update(model),
        Message::UnstageSelected => unstage_selected::update(model),
        Message::UnstageAll => unstage_all::update(model),
        Message::DiscardSelected => discard_selected::update(model),
        Message::ConfirmDiscard(target) => confirm_discard::update(model, target),
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
            show_rename_branch_input::update(model, old_name)
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
        Message::ShowSelectDialog(show_select) => match show_select {
            SelectDialog::FetchUpstream => show_fetch_upstream_select::update(model),
            SelectDialog::FetchElsewhere => show_fetch_elsewhere_select::update(model),
            SelectDialog::FetchAnotherBranch => show_fetch_another_branch_select::update(model),
            SelectDialog::FetchAnotherBranchBranch(remote) => {
                show_fetch_another_branch_branch_select::update(model, remote)
            }
            SelectDialog::FetchPushRemote => show_fetch_push_remote_select::update(model),
            SelectDialog::PushUpstream => show_push_upstream_select::update(model),
            SelectDialog::PushPushRemote => show_push_push_remote_select::update(model),
            SelectDialog::PushAllTags => show_push_all_tags_select::update(model),
            SelectDialog::PushTag => show_push_tag_select::update(model),
            SelectDialog::PullUpstream => show_pull_upstream_select::update(model),
            SelectDialog::PullPushRemote => show_pull_push_remote_select::update(model),
            SelectDialog::CheckoutBranch => show_checkout_branch_popup::update(model),
            SelectDialog::CheckoutLocalBranch => show_checkout_local_branch_popup::update(model),
            SelectDialog::DeleteBranch => show_delete_branch_popup::update(model),
            SelectDialog::RenameBranch => show_rename_branch_popup::update(model),
            SelectDialog::CreateNewBranch { checkout } => {
                show_checkout_new_branch_popup::update(model, checkout)
            }
            SelectDialog::StashApply => show_stash_apply_select::update(model),
            SelectDialog::StashDrop => show_stash_drop_select::update(model),
            SelectDialog::FixupCommit(fixup_type) => {
                show_fixup_commit_select::update(model, fixup_type)
            }
            SelectDialog::OpenPr => show_open_pr_select::update(model, false),
            SelectDialog::OpenPrWithTarget => show_open_pr_select::update(model, true),
            SelectDialog::OpenPrTarget(branch) => show_open_pr_target_select::update(model, branch),
        },
        Message::EnterArgMode => enter_arg_mode::update(model),
        Message::ExitArgMode => exit_arg_mode::update(model),
        Message::ToggleArgument(argument) => toggle_argument::update(model, argument),
        Message::Select(select_msg) => match select_msg {
            SelectMessage::Show { title, options } => {
                show_select_popup::update(model, title, options)
            }
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

        Message::Fetch(fetch_command) => fetch::update(model, fetch_command),
        Message::Pull(pull_command) => pull::update(model, pull_command),
        Message::Stash(stash_command) => stash::update(model, stash_command),
        Message::Push(push_command) => push::update(model, push_command),
    }
}
