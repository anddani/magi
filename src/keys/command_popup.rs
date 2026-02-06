use crate::{model::arguments::PushArgument, msg::SelectMessage};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::{model::popup::PopupContentCommand, msg::Message};

mod push;

pub fn handle_command_popup_key(
    key: KeyEvent,
    command: &PopupContentCommand,
    arg_mode: bool,
) -> Option<Message> {
    match command {
        PopupContentCommand::Help => match (key.modifiers, key.code) {
            (KeyModifiers::NONE, KeyCode::Esc | KeyCode::Char('q'))
            | (KeyModifiers::CONTROL, KeyCode::Char('g')) => Some(Message::DismissPopup),
            _ => None,
        },
        PopupContentCommand::Commit => match (key.modifiers, key.code) {
            (KeyModifiers::NONE, KeyCode::Esc | KeyCode::Char('q'))
            | (KeyModifiers::CONTROL, KeyCode::Char('g')) => Some(Message::DismissPopup),
            (KeyModifiers::NONE, KeyCode::Char('c')) => Some(Message::Commit),
            (KeyModifiers::NONE, KeyCode::Char('a')) => Some(Message::Amend),
            _ => None,
        },
        PopupContentCommand::Branch => match (key.modifiers, key.code) {
            (KeyModifiers::NONE, KeyCode::Esc | KeyCode::Char('q'))
            | (KeyModifiers::CONTROL, KeyCode::Char('g')) => Some(Message::DismissPopup),
            (KeyModifiers::NONE, KeyCode::Char('b')) => Some(Message::ShowCheckoutBranchPopup),
            _ => None,
        },
        PopupContentCommand::Push(state) => push::keys(key, arg_mode, state),
        PopupContentCommand::Select(_) => match (key.modifiers, key.code) {
            (KeyModifiers::NONE, KeyCode::Esc) | (KeyModifiers::CONTROL, KeyCode::Char('g')) => {
                Some(Message::DismissPopup)
            }
            (KeyModifiers::NONE, KeyCode::Enter) => Some(Message::Select(SelectMessage::Confirm)),
            (KeyModifiers::NONE, KeyCode::Backspace) => {
                Some(Message::Select(SelectMessage::InputBackspace))
            }
            (KeyModifiers::NONE, KeyCode::Up) | (KeyModifiers::CONTROL, KeyCode::Char('p')) => {
                Some(Message::Select(SelectMessage::MoveUp))
            }
            (KeyModifiers::NONE, KeyCode::Down) | (KeyModifiers::CONTROL, KeyCode::Char('n')) => {
                Some(Message::Select(SelectMessage::MoveDown))
            }
            (KeyModifiers::NONE, KeyCode::Char(c)) | (KeyModifiers::SHIFT, KeyCode::Char(c)) => {
                Some(Message::Select(SelectMessage::InputChar(c)))
            }
            _ => None,
        },
    }
}
