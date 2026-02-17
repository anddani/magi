use crossterm::event::{KeyCode, KeyEvent};

use crate::msg::Message;

pub fn keys(key: KeyEvent) -> Option<Message> {
    match key.code {
        KeyCode::Char('q') => Some(Message::DismissPopup),
        KeyCode::Char('b') => Some(Message::ShowCheckoutBranchPopup),
        KeyCode::Char('l') => Some(Message::ShowCheckoutLocalBranchPopup),
        KeyCode::Char('c') => Some(Message::ShowCreateNewBranchPopup { checkout: true }),
        KeyCode::Char('n') => Some(Message::ShowCreateNewBranchPopup { checkout: false }),
        KeyCode::Char('x') => Some(Message::ShowDeleteBranchPopup),
        KeyCode::Char('o') => Some(Message::ShowOpenPrSelect),
        KeyCode::Char('O') => Some(Message::ShowOpenPrWithTargetSelect),
        _ => None,
    }
}
