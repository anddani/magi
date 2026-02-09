use crate::{
    git::credential::CredentialResponse,
    model::{
        popup::{PopupContent, PopupContentCommand, SelectResult},
        Model,
    },
    msg::Message,
};

pub fn update(model: &mut Model) -> Option<Message> {
    // If dismissing a select popup, mark it as cancelled
    if let Some(PopupContent::Command(PopupContentCommand::Select(_))) = &model.popup {
        model.select_result = Some(SelectResult::Cancelled);
    }

    // If dismissing a credential popup, send cancelled response to PTY thread
    if let Some(PopupContent::Credential(_)) = &model.popup {
        if let Some(ref pty_state) = model.pty_state {
            let _ = pty_state.send_credential(CredentialResponse::Cancelled);
        }
    }

    // If dismissing a push popup, reset the arguments
    model.arguments = None;
    model.popup = None;
    None
}
