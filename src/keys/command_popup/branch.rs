use crossterm::event::{KeyCode, KeyEvent};

use crate::msg::{Message, ShowSelectDialog};

pub fn keys(key: KeyEvent) -> Option<Message> {
    match key.code {
        KeyCode::Char('q') => Some(Message::DismissPopup),
        KeyCode::Char('b') => Some(Message::ShowSelect(ShowSelectDialog::CheckoutBranch)),
        KeyCode::Char('l') => Some(Message::ShowSelect(ShowSelectDialog::CheckoutLocalBranch)),
        KeyCode::Char('c') => Some(Message::ShowSelect(ShowSelectDialog::CreateNewBranch {
            checkout: true,
        })),
        KeyCode::Char('n') => Some(Message::ShowSelect(ShowSelectDialog::CreateNewBranch {
            checkout: false,
        })),
        KeyCode::Char('m') => Some(Message::ShowSelect(ShowSelectDialog::RenameBranch)),
        KeyCode::Char('x') => Some(Message::ShowSelect(ShowSelectDialog::DeleteBranch)),
        KeyCode::Char('o') => Some(Message::ShowSelect(ShowSelectDialog::OpenPr)),
        KeyCode::Char('O') => Some(Message::ShowSelect(ShowSelectDialog::OpenPrWithTarget)),
        _ => None,
    }
}
