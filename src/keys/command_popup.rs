use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::{model::popup::PopupContentCommand, msg::Message};

mod apply;
mod branch;
mod commit;
mod configure;
mod fetch;
mod log;
mod merge;
mod multi_select;
mod pull;
mod push;
mod rebase;
mod reset;
mod revert;
mod select;
mod stash;
mod stash_push;
mod tag;
mod worktree;

pub fn handle_command_popup_key(
    key: KeyEvent,
    command: &PopupContentCommand,
    arg_mode: bool,
    equals_arg_mode: bool,
) -> Option<Message> {
    if key.code == KeyCode::Esc
        || key.modifiers == KeyModifiers::CONTROL && key.code == KeyCode::Char('g')
        || key.modifiers == KeyModifiers::CONTROL && key.code == KeyCode::Char('c')
    {
        return Some(Message::DismissPopup);
    }

    match command {
        PopupContentCommand::Commit(_) => commit::keys(key, arg_mode),
        PopupContentCommand::Branch => branch::keys(key),
        PopupContentCommand::Configure(state) => configure::keys(key, state),
        PopupContentCommand::Fetch(state) => fetch::keys(key, arg_mode, state),
        PopupContentCommand::Log => log::keys(key, arg_mode, equals_arg_mode),
        PopupContentCommand::Pull(state) => pull::keys(key, arg_mode, state),
        PopupContentCommand::Push(state) => push::keys(key, arg_mode, state),
        PopupContentCommand::Stash => stash::keys(key, arg_mode),
        PopupContentCommand::StashPush => stash_push::keys(key, arg_mode),
        PopupContentCommand::Reset => reset::keys(key),
        PopupContentCommand::Rebase(state) => rebase::keys(key, arg_mode, state),
        PopupContentCommand::Revert(state) => revert::keys(key, arg_mode, equals_arg_mode, state),
        PopupContentCommand::Merge(state) => merge::keys(key, arg_mode, state),
        PopupContentCommand::Apply(state) => apply::keys(key, state),
        PopupContentCommand::Tag => tag::keys(key, arg_mode),
        PopupContentCommand::Worktree => worktree::keys(key),
        PopupContentCommand::Select(_) => select::keys(key),
        PopupContentCommand::MultiSelect(_) => multi_select::keys(key),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::popup::{FetchPopupState, PullPopupState, PushPopupState};

    fn c_key() -> KeyEvent {
        KeyEvent::new(KeyCode::Char('C'), KeyModifiers::NONE)
    }

    fn popup_state() -> (FetchPopupState, PullPopupState, PushPopupState) {
        let fetch = FetchPopupState {
            upstream: None,
            push_remote: None,
            sole_remote: None,
        };
        let pull = PullPopupState {
            upstream: None,
            push_remote: None,
            sole_remote: None,
        };
        let push = PushPopupState {
            upstream: None,
            push_remote: None,
            sole_remote: None,
        };
        (fetch, pull, push)
    }

    #[test]
    fn c_opens_configure_without_force_pick_in_remote_popups() {
        let (fetch, pull, push) = popup_state();
        for cmd in [
            PopupContentCommand::Fetch(fetch),
            PopupContentCommand::Pull(pull),
            PopupContentCommand::Push(push),
        ] {
            assert_eq!(
                handle_command_popup_key(c_key(), &cmd, false, false),
                Some(Message::ShowConfigurePopup { force_pick: false }),
                "C in {:?} should open Configure for the current branch",
                cmd
            );
        }
    }

    #[test]
    fn c_opens_configure_with_force_pick_in_branch_popup() {
        assert_eq!(
            handle_command_popup_key(c_key(), &PopupContentCommand::Branch, false, false),
            Some(Message::ShowConfigurePopup { force_pick: true }),
        );
    }
}
