use crossterm::event::{KeyCode, KeyEvent};

use crate::msg::{Message, SelectPopup};

pub fn keys(key: KeyEvent) -> Option<Message> {
    match key.code {
        KeyCode::Char('q') => Some(Message::DismissPopup),
        KeyCode::Char('b') => Some(Message::ShowSelectPopup(SelectPopup::CheckoutBranch)),
        KeyCode::Char('l') => Some(Message::ShowSelectPopup(SelectPopup::CheckoutLocalBranch)),
        KeyCode::Char('c') => Some(Message::ShowSelectPopup(SelectPopup::CreateNewBranch {
            checkout: true,
        })),
        KeyCode::Char('n') => Some(Message::ShowSelectPopup(SelectPopup::CreateNewBranch {
            checkout: false,
        })),
        KeyCode::Char('s') => Some(Message::ShowSpinoffBranchInput),
        KeyCode::Char('S') => Some(Message::ShowSpinoutBranchInput),
        KeyCode::Char('w') => Some(Message::ShowSelectPopup(SelectPopup::WorktreeCheckout)),
        KeyCode::Char('W') => Some(Message::ShowSelectPopup(SelectPopup::WorktreeCreate)),
        KeyCode::Char('m') => Some(Message::ShowSelectPopup(SelectPopup::RenameBranch)),
        KeyCode::Char('x') => Some(Message::ShowSelectPopup(SelectPopup::DeleteBranch)),
        KeyCode::Char('X') => Some(Message::ShowSelectPopup(SelectPopup::ResetBranchPick)),
        KeyCode::Char('o') => Some(Message::ShowSelectPopup(SelectPopup::OpenPr)),
        KeyCode::Char('O') => Some(Message::ShowSelectPopup(SelectPopup::OpenPrWithTarget)),
        _ => None,
    }
}
