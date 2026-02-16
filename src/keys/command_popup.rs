use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::{model::popup::PopupContentCommand, msg::Message};

mod branch;
mod commit;
mod fetch;
mod log;
mod pull;
mod push;
mod select;

pub fn handle_command_popup_key(
    key: KeyEvent,
    command: &PopupContentCommand,
    arg_mode: bool,
) -> Option<Message> {
    if key.code == KeyCode::Esc
        || key.modifiers == KeyModifiers::CONTROL && key.code == KeyCode::Char('g')
        || key.modifiers == KeyModifiers::CONTROL && key.code == KeyCode::Char('c')
    {
        return Some(Message::DismissPopup);
    }

    match command {
        PopupContentCommand::Commit => commit::keys(key, arg_mode),
        PopupContentCommand::Branch => branch::keys(key),
        PopupContentCommand::Fetch(state) => fetch::keys(key, arg_mode, state),
        PopupContentCommand::Log => log::keys(key),
        PopupContentCommand::Pull(state) => pull::keys(key, arg_mode, state),
        PopupContentCommand::Push(state) => push::keys(key, arg_mode, state),
        PopupContentCommand::Select(_) => select::keys(key),
    }
}
