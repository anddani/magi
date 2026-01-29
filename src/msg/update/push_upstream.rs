use crate::{
    git::{credential::CredentialStrategy, pty_command::spawn_git_with_pty},
    model::{
        popup::{PopupContent, PopupContentCommand},
        Model, PtyState,
    },
    msg::Message,
};

pub fn update(model: &mut Model) -> Option<Message> {
    // Check if force_with_lease is enabled before dismissing popup
    let force_with_lease =
        if let Some(PopupContent::Command(PopupContentCommand::Push(ref state))) = model.popup {
            state.force_with_lease
        } else {
            false
        };

    model.popup = None;

    if model.pty_state.is_some() {
        model.popup = Some(PopupContent::Error {
            message: "A command is already in progress".to_string(),
        });
        return None;
    }

    let Some(repo_path) = model.git_info.repository.workdir() else {
        model.popup = Some(PopupContent::Error {
            message: "Repository working directory not found".into(),
        });
        return None;
    };

    // Build push command arguments
    let mut args = vec!["push".to_string(), "-v".to_string()];
    if force_with_lease {
        args.push("--force-with-lease".to_string());
    }

    // Spawn push command in background thread with PTY
    let (result_rx, ui_channels) =
        spawn_git_with_pty(repo_path.to_path_buf(), args, CredentialStrategy::Prompt);

    // Store PTY state for main loop to monitor
    if let Some(ui_channels) = ui_channels {
        model.pty_state = Some(PtyState::new(
            result_rx,
            ui_channels.request_rx,
            ui_channels.response_tx,
            "Push".to_string(),
        ));
    } else {
        // This shouldn't happen with Prompt strategy, but handle it
        model.popup = Some(PopupContent::Error {
            message: "Failed to initialize credential handling".to_string(),
        });
    }

    // Don't refresh yet - wait for command to complete
    None
}
