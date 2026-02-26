use crossterm::event::{KeyCode, KeyEvent};

use crate::msg::{Message, SelectDialog};

pub fn keys(key: KeyEvent) -> Option<Message> {
    match key.code {
        KeyCode::Char('q') => Some(Message::DismissPopup),
        KeyCode::Char('b') => Some(Message::ShowSelectDialog(SelectDialog::CheckoutBranch)),
        KeyCode::Char('l') => Some(Message::ShowSelectDialog(SelectDialog::CheckoutLocalBranch)),
        KeyCode::Char('c') => Some(Message::ShowSelectDialog(SelectDialog::CreateNewBranch {
            checkout: true,
        })),
        KeyCode::Char('n') => Some(Message::ShowSelectDialog(SelectDialog::CreateNewBranch {
            checkout: false,
        })),
        KeyCode::Char('m') => Some(Message::ShowSelectDialog(SelectDialog::RenameBranch)),
        KeyCode::Char('x') => Some(Message::ShowSelectDialog(SelectDialog::DeleteBranch)),
        KeyCode::Char('o') => Some(Message::ShowSelectDialog(SelectDialog::OpenPr)),
        KeyCode::Char('O') => Some(Message::ShowSelectDialog(SelectDialog::OpenPrWithTarget)),
        _ => None,
    }
}
