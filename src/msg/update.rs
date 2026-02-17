use crate::{
    model::Model,
    msg::{InputMessage, Message, SelectMessage},
};

mod amend;
mod checkout_branch;
mod checkout_new_branch;
mod commit;
mod confirm_delete_branch;
mod credentials_input;
mod delete_branch;
mod dismiss_popup;
mod enter_arg_mode;
mod enter_visual_mode;
mod exit_arg_mode;
mod exit_log_view;
mod exit_visual_mode;
mod fetch_all_remotes;
mod fetch_from_remote;
mod fetch_upstream;
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
mod pull_from_remote;
mod pull_upstream;
mod push_all_tags;
mod push_helper;
mod push_tag;
mod push_to_remote;
mod push_upstream;
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
mod show_checkout_branch_popup;
mod show_checkout_local_branch_popup;
mod show_checkout_new_branch_input;
mod show_checkout_new_branch_popup;
mod show_delete_branch_popup;
mod show_fetch_elsewhere_select;
mod show_fetch_popup;
mod show_fetch_upstream_select;
mod show_log;
mod show_open_pr_select;
mod show_open_pr_target_select;
mod show_pull_popup;
mod show_pull_upstream_select;
mod show_push_all_tags_select;
mod show_push_popup;
mod show_push_tag_select;
mod show_push_upstream_select;
mod show_rename_branch_input;
mod show_rename_branch_popup;
mod show_select_popup;
mod stage_all_modified;
mod stage_selected;
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
        Message::DismissPopup => dismiss_popup::update(model),
        Message::StageAllModified => stage_all_modified::update(model),
        Message::StageSelected => stage_selected::update(model),
        Message::UnstageSelected => unstage_selected::update(model),
        Message::UnstageAll => unstage_all::update(model),
        Message::EnterVisualMode => enter_visual_mode::update(model),
        Message::ExitVisualMode => exit_visual_mode::update(model),
        Message::ShowPopup(content) => {
            model.popup = Some(content);
            None
        }
        Message::ShowPushPopup => show_push_popup::update(model),
        Message::ShowFetchPopup => show_fetch_popup::update(model),
        Message::ShowCheckoutBranchPopup => show_checkout_branch_popup::update(model),
        Message::ShowCheckoutLocalBranchPopup => show_checkout_local_branch_popup::update(model),
        Message::ShowDeleteBranchPopup => show_delete_branch_popup::update(model),
        Message::ShowRenameBranchPopup => show_rename_branch_popup::update(model),
        Message::ShowRenameBranchInput(old_name) => {
            show_rename_branch_input::update(model, old_name)
        }
        Message::RenameBranch { old_name, new_name } => {
            rename_branch::update(model, old_name, new_name)
        }
        Message::ShowCreateNewBranchPopup { checkout } => {
            show_checkout_new_branch_popup::update(model, checkout)
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
        Message::FetchAllRemotes => fetch_all_remotes::update(model),
        Message::FetchUpstream => fetch_upstream::update(model),
        Message::ShowFetchUpstreamSelect => show_fetch_upstream_select::update(model),
        Message::ShowFetchElsewhereSelect => show_fetch_elsewhere_select::update(model),
        Message::FetchFromRemote(upstream) => fetch_from_remote::update(model, upstream),
        Message::PushUpstream => push_upstream::update(model),
        Message::ShowPushUpstreamSelect => show_push_upstream_select::update(model),
        Message::PushToRemote(upstream) => push_to_remote::update(model, upstream),
        Message::ShowPushAllTagsSelect => show_push_all_tags_select::update(model),
        Message::PushAllTags(remote) => push_all_tags::update(model, remote),
        Message::ShowPushTagSelect => show_push_tag_select::update(model),
        Message::PushTag(tag) => push_tag::update(model, tag),
        Message::ShowPullPopup => show_pull_popup::update(model),
        Message::PullUpstream => pull_upstream::update(model),
        Message::ShowPullUpstreamSelect => show_pull_upstream_select::update(model),
        Message::PullFromRemote(upstream) => pull_from_remote::update(model, upstream),
        Message::ShowOpenPrSelect => show_open_pr_select::update(model, false),
        Message::ShowOpenPrWithTargetSelect => show_open_pr_select::update(model, true),
        Message::ShowOpenPrTargetSelect(branch) => {
            show_open_pr_target_select::update(model, branch)
        }
        Message::OpenPr { branch, target } => open_pr::update(model, branch, target),
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
    }
}
