use crossterm::event::{KeyCode, KeyEvent};

use crate::{
    model::arguments::{Argument::StashPush, StashPushArgument},
    msg::Message,
};

/// Keys for the nested "Stash push" popup. `-` enters argument mode where
/// `u`/`a`/`k`/`K` toggle the flags; `P` opens the file picker.
pub fn keys(key: KeyEvent, arg_mode: bool) -> Option<Message> {
    if arg_mode {
        return match key.code {
            KeyCode::Char(c) => StashPushArgument::from_key(c)
                .map(|arg| Message::ToggleArgument(StashPush(arg)))
                .or(Some(Message::ExitArgMode)),
            _ => Some(Message::ExitArgMode),
        };
    }

    match key.code {
        KeyCode::Char('P') => Some(Message::ShowStashPushFiles),
        KeyCode::Char('-') => Some(Message::EnterArgMode),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::KeyModifiers;

    fn key(c: char) -> KeyEvent {
        KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE)
    }

    #[test]
    fn test_p_opens_file_picker() {
        assert_eq!(keys(key('P'), false), Some(Message::ShowStashPushFiles));
    }

    #[test]
    fn test_dash_enters_arg_mode() {
        assert_eq!(keys(key('-'), false), Some(Message::EnterArgMode));
    }

    #[test]
    fn test_arg_mode_toggles_keep_index() {
        assert_eq!(
            keys(key('k'), true),
            Some(Message::ToggleArgument(StashPush(
                StashPushArgument::KeepIndex
            )))
        );
        assert_eq!(
            keys(key('K'), true),
            Some(Message::ToggleArgument(StashPush(
                StashPushArgument::NoKeepIndex
            )))
        );
    }

    #[test]
    fn test_arg_mode_toggles_untracked_and_all() {
        assert_eq!(
            keys(key('u'), true),
            Some(Message::ToggleArgument(StashPush(
                StashPushArgument::IncludeUntracked
            )))
        );
        assert_eq!(
            keys(key('a'), true),
            Some(Message::ToggleArgument(StashPush(StashPushArgument::All)))
        );
    }

    #[test]
    fn test_arg_mode_unknown_key_exits() {
        assert_eq!(keys(key('z'), true), Some(Message::ExitArgMode));
    }

    #[test]
    fn test_unknown_key_does_nothing() {
        assert_eq!(keys(key('q'), false), None);
    }
}
