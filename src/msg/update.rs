use crate::{
    model::Model,
    msg::{Message, SelectMessage},
};

mod amend;
mod checkout_branch;
mod commit;
mod credentials_input;
mod dismiss_popup;
mod enter_visual_mode;
mod exit_visual_mode;
mod half_page_down;
mod half_page_up;
mod move_down;
mod move_up;
mod push_confirm_input;
mod push_enter_arg_mode;
mod push_enter_input_mode;
mod push_exit_arg_mode;
mod push_input_backspace;
mod push_input_char;
mod push_input_complete;
mod push_toggle_force_with_lease;
mod push_upstream;
mod quit;
mod refresh;
mod scroll_line_down;
mod scroll_line_up;
mod select_confirm;
mod select_input_backspace;
mod select_input_char;
mod select_move_down;
mod select_move_up;
mod show_branch_popup;
mod show_checkout_branch_popup;
mod show_commit_popup;
mod show_help;
mod show_push_popup;
mod show_select_popup;
mod stage_all_modified;
mod toggle_section;
mod unstage_all;

/// Processes a [`Message`], modifying the passed model.
///
/// Returns a follow up [`Message`] for sequences of actions.
/// e.g. after a stage, a [`Message::Refresh`] should be triggered.
pub fn update(model: &mut Model, msg: Message) -> Option<Message> {
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
        Message::Commit => commit::update(model),
        Message::Amend => amend::update(model),
        Message::DismissPopup => dismiss_popup::update(model),
        Message::StageAllModified => stage_all_modified::update(model),
        Message::UnstageAll => unstage_all::update(model),
        Message::EnterVisualMode => enter_visual_mode::update(model),
        Message::ExitVisualMode => exit_visual_mode::update(model),
        Message::ShowHelp => show_help::update(model),
        Message::ShowCommitPopup => show_commit_popup::update(model),
        Message::ShowPushPopup => show_push_popup::update(model),
        Message::ShowBranchPopup => show_branch_popup::update(model),
        Message::ShowCheckoutBranchPopup => show_checkout_branch_popup::update(model),
        Message::CheckoutBranch(branch) => checkout_branch::update(model, branch),
        Message::PushUpstream => push_upstream::update(model),
        Message::PushEnterInputMode => push_enter_input_mode::update(model),
        Message::PushInputChar(c) => push_input_char::update(model, c),
        Message::PushInputBackspace => push_input_backspace::update(model),
        Message::PushInputComplete => push_input_complete::update(model),
        Message::PushConfirmInput => push_confirm_input::update(model),
        Message::PushEnterArgMode => push_enter_arg_mode::update(model),
        Message::PushExitArgMode => push_exit_arg_mode::update(model),
        Message::PushToggleForceWithLease => push_toggle_force_with_lease::update(model),
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
        Message::Credentials(credentials_msg) => credentials_input::update(model, credentials_msg),
    }
}
