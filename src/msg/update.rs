use crate::{model::Model, msg::Message};

mod commit;
mod dismiss_dialog;
mod enter_visual_mode;
mod exit_visual_mode;
mod half_page_down;
mod half_page_up;
mod move_down;
mod move_up;
mod quit;
mod refresh;
mod scroll_line_down;
mod scroll_line_up;
mod show_help;
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
        Message::DismissDialog => dismiss_dialog::update(model),
        Message::StageAllModified => stage_all_modified::update(model),
        Message::UnstageAll => unstage_all::update(model),
        Message::EnterVisualMode => enter_visual_mode::update(model),
        Message::ExitVisualMode => exit_visual_mode::update(model),
        Message::ShowHelp => show_help::update(model),
    }
}
