use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::model::EditOp;

const NONE: KeyModifiers = KeyModifiers::NONE;
const SHIFT: KeyModifiers = KeyModifiers::SHIFT;
const CTRL: KeyModifiers = KeyModifiers::CONTROL;
const ALT: KeyModifiers = KeyModifiers::ALT;
const SUPER: KeyModifiers = KeyModifiers::SUPER;

/// Maps a key event to a readline-style text editing operation.
/// Shared by every text input surface so the bindings are defined once.
/// Ctrl+g is deliberately unbound (it cancels popups everywhere).
pub fn edit_op_for_key(key: KeyEvent) -> Option<EditOp> {
    // Backspace with modifiers: some terminals report Alt+Backspace as
    // ALT|CONTROL, so match on containment rather than exact equality.
    if key.code == KeyCode::Backspace {
        if key.modifiers.contains(SUPER) {
            return Some(EditOp::DeleteToStart);
        }
        if key.modifiers.contains(ALT) || key.modifiers.contains(CTRL) {
            return Some(EditOp::DeleteWordBackward);
        }
        return Some(EditOp::DeleteBackward);
    }

    match (key.modifiers, key.code) {
        (CTRL, KeyCode::Char('h')) => Some(EditOp::DeleteBackward),
        (NONE, KeyCode::Delete) | (CTRL, KeyCode::Char('d')) => Some(EditOp::DeleteForward),
        (CTRL, KeyCode::Char('w')) => Some(EditOp::DeleteWordBackward),
        (ALT, KeyCode::Char('d')) => Some(EditOp::DeleteWordForward),
        (CTRL, KeyCode::Char('u')) => Some(EditOp::DeleteToStart),
        (CTRL, KeyCode::Char('k')) => Some(EditOp::DeleteToEnd),
        (NONE, KeyCode::Left) | (CTRL, KeyCode::Char('b')) => Some(EditOp::MoveLeft),
        (NONE, KeyCode::Right) | (CTRL, KeyCode::Char('f')) => Some(EditOp::MoveRight),
        (ALT, KeyCode::Left) | (ALT, KeyCode::Char('b')) => Some(EditOp::MoveWordLeft),
        (ALT, KeyCode::Right) | (ALT, KeyCode::Char('f')) => Some(EditOp::MoveWordRight),
        (NONE, KeyCode::Home) | (CTRL, KeyCode::Char('a')) => Some(EditOp::MoveToStart),
        (NONE, KeyCode::End) | (CTRL, KeyCode::Char('e')) => Some(EditOp::MoveToEnd),
        (NONE, KeyCode::Char(c)) | (SHIFT, KeyCode::Char(c)) => Some(EditOp::Insert(c)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyEventKind, KeyEventState};

    fn key(modifiers: KeyModifiers, code: KeyCode) -> KeyEvent {
        KeyEvent {
            code,
            modifiers,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }
    }

    #[test]
    fn insert_plain_and_shifted_chars() {
        assert_eq!(
            edit_op_for_key(key(NONE, KeyCode::Char('a'))),
            Some(EditOp::Insert('a'))
        );
        assert_eq!(
            edit_op_for_key(key(SHIFT, KeyCode::Char('A'))),
            Some(EditOp::Insert('A'))
        );
    }

    #[test]
    fn delete_backward_bindings() {
        assert_eq!(
            edit_op_for_key(key(NONE, KeyCode::Backspace)),
            Some(EditOp::DeleteBackward)
        );
        assert_eq!(
            edit_op_for_key(key(CTRL, KeyCode::Char('h'))),
            Some(EditOp::DeleteBackward)
        );
    }

    #[test]
    fn delete_forward_bindings() {
        assert_eq!(
            edit_op_for_key(key(NONE, KeyCode::Delete)),
            Some(EditOp::DeleteForward)
        );
        assert_eq!(
            edit_op_for_key(key(CTRL, KeyCode::Char('d'))),
            Some(EditOp::DeleteForward)
        );
    }

    #[test]
    fn delete_word_backward_bindings() {
        assert_eq!(
            edit_op_for_key(key(ALT, KeyCode::Backspace)),
            Some(EditOp::DeleteWordBackward)
        );
        assert_eq!(
            edit_op_for_key(key(CTRL, KeyCode::Char('w'))),
            Some(EditOp::DeleteWordBackward)
        );
        assert_eq!(
            edit_op_for_key(key(CTRL, KeyCode::Backspace)),
            Some(EditOp::DeleteWordBackward)
        );
        // Some terminals report Alt+Backspace with combined modifiers
        assert_eq!(
            edit_op_for_key(key(ALT | CTRL, KeyCode::Backspace)),
            Some(EditOp::DeleteWordBackward)
        );
    }

    #[test]
    fn delete_word_forward_binding() {
        assert_eq!(
            edit_op_for_key(key(ALT, KeyCode::Char('d'))),
            Some(EditOp::DeleteWordForward)
        );
    }

    #[test]
    fn delete_to_start_bindings() {
        assert_eq!(
            edit_op_for_key(key(SUPER, KeyCode::Backspace)),
            Some(EditOp::DeleteToStart)
        );
        assert_eq!(
            edit_op_for_key(key(CTRL, KeyCode::Char('u'))),
            Some(EditOp::DeleteToStart)
        );
    }

    #[test]
    fn delete_to_end_binding() {
        assert_eq!(
            edit_op_for_key(key(CTRL, KeyCode::Char('k'))),
            Some(EditOp::DeleteToEnd)
        );
    }

    #[test]
    fn char_movement_bindings() {
        assert_eq!(
            edit_op_for_key(key(NONE, KeyCode::Left)),
            Some(EditOp::MoveLeft)
        );
        assert_eq!(
            edit_op_for_key(key(CTRL, KeyCode::Char('b'))),
            Some(EditOp::MoveLeft)
        );
        assert_eq!(
            edit_op_for_key(key(NONE, KeyCode::Right)),
            Some(EditOp::MoveRight)
        );
        assert_eq!(
            edit_op_for_key(key(CTRL, KeyCode::Char('f'))),
            Some(EditOp::MoveRight)
        );
    }

    #[test]
    fn word_movement_bindings() {
        assert_eq!(
            edit_op_for_key(key(ALT, KeyCode::Left)),
            Some(EditOp::MoveWordLeft)
        );
        assert_eq!(
            edit_op_for_key(key(ALT, KeyCode::Char('b'))),
            Some(EditOp::MoveWordLeft)
        );
        assert_eq!(
            edit_op_for_key(key(ALT, KeyCode::Right)),
            Some(EditOp::MoveWordRight)
        );
        assert_eq!(
            edit_op_for_key(key(ALT, KeyCode::Char('f'))),
            Some(EditOp::MoveWordRight)
        );
    }

    #[test]
    fn start_end_bindings() {
        assert_eq!(
            edit_op_for_key(key(NONE, KeyCode::Home)),
            Some(EditOp::MoveToStart)
        );
        assert_eq!(
            edit_op_for_key(key(CTRL, KeyCode::Char('a'))),
            Some(EditOp::MoveToStart)
        );
        assert_eq!(
            edit_op_for_key(key(NONE, KeyCode::End)),
            Some(EditOp::MoveToEnd)
        );
        assert_eq!(
            edit_op_for_key(key(CTRL, KeyCode::Char('e'))),
            Some(EditOp::MoveToEnd)
        );
    }

    #[test]
    fn unbound_keys_map_to_none() {
        assert_eq!(edit_op_for_key(key(CTRL, KeyCode::Char('g'))), None);
        assert_eq!(edit_op_for_key(key(NONE, KeyCode::Enter)), None);
        assert_eq!(edit_op_for_key(key(NONE, KeyCode::Esc)), None);
        assert_eq!(edit_op_for_key(key(CTRL, KeyCode::Char('c'))), None);
    }
}
