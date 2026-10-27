use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::keys::edit::edit_op_for_key;
use crate::msg::{Message, MultiSelectMessage};

/// Keys for the multi-select picker. SPACE toggles the item under the cursor
/// (so it never reaches the filter), RET confirms, and everything else drives
/// the fuzzy filter just like the single-select picker.
pub fn keys(key: KeyEvent) -> Option<Message> {
    match (key.modifiers, key.code) {
        (KeyModifiers::NONE, KeyCode::Enter) => {
            Some(Message::MultiSelect(MultiSelectMessage::Confirm))
        }
        (KeyModifiers::NONE, KeyCode::Char(' ')) => {
            Some(Message::MultiSelect(MultiSelectMessage::ToggleCurrent))
        }
        (KeyModifiers::NONE, KeyCode::Up) | (KeyModifiers::CONTROL, KeyCode::Char('p')) => {
            Some(Message::MultiSelect(MultiSelectMessage::MoveUp))
        }
        (KeyModifiers::NONE, KeyCode::Down) | (KeyModifiers::CONTROL, KeyCode::Char('n')) => {
            Some(Message::MultiSelect(MultiSelectMessage::MoveDown))
        }
        _ => edit_op_for_key(key).map(|op| Message::MultiSelect(MultiSelectMessage::Edit(op))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::EditOp;

    fn key(modifiers: KeyModifiers, code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, modifiers)
    }

    #[test]
    fn test_space_toggles_current() {
        assert_eq!(
            keys(key(KeyModifiers::NONE, KeyCode::Char(' '))),
            Some(Message::MultiSelect(MultiSelectMessage::ToggleCurrent))
        );
    }

    #[test]
    fn test_enter_confirms() {
        assert_eq!(
            keys(key(KeyModifiers::NONE, KeyCode::Enter)),
            Some(Message::MultiSelect(MultiSelectMessage::Confirm))
        );
    }

    #[test]
    fn test_arrows_move() {
        assert_eq!(
            keys(key(KeyModifiers::NONE, KeyCode::Up)),
            Some(Message::MultiSelect(MultiSelectMessage::MoveUp))
        );
        assert_eq!(
            keys(key(KeyModifiers::NONE, KeyCode::Down)),
            Some(Message::MultiSelect(MultiSelectMessage::MoveDown))
        );
    }

    #[test]
    fn test_char_filters() {
        assert_eq!(
            keys(key(KeyModifiers::NONE, KeyCode::Char('a'))),
            Some(Message::MultiSelect(MultiSelectMessage::Edit(
                EditOp::Insert('a')
            )))
        );
    }
}
