use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::{model::popup::PopupContentCommand, msg::Message};

mod branch;
mod commit;
mod help;
mod push;
mod select;

pub fn handle_command_popup_key(
    key: KeyEvent,
    command: &PopupContentCommand,
    arg_mode: bool,
) -> Option<Message> {
    if key.code == KeyCode::Esc
        || key.modifiers == KeyModifiers::CONTROL && key.code == KeyCode::Char('g')
    {
        return Some(Message::DismissPopup);
    }

    match command {
        PopupContentCommand::Help => help::keys(key),
        PopupContentCommand::Commit => commit::keys(key),
        PopupContentCommand::Branch => branch::keys(key),
        PopupContentCommand::Push(state) => push::keys(key, arg_mode, state),
        PopupContentCommand::Select(_) => select::keys(key),
    }
}
