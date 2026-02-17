use crate::{
    git::{credential::CredentialStrategy, pty_command::spawn_git_with_pty},
    model::{Model, PtyState, popup::PopupContent},
    msg::Message,
};

/// Execute a git command with PTY support for credential handling.
///
/// This handles the common PTY execution logic:
/// - Checking for existing PTY state
/// - Getting the repository path
/// - Spawning the PTY command
/// - Setting up PtyState or showing error popup
///
/// # Arguments
/// * `model` - The application model
/// * `args` - Arguments to pass to git (e.g., `["fetch", "-v", "--all"]`)
/// * `operation_name` - Name to display for this operation (e.g., "Fetch all")
pub fn execute_pty_command(
    model: &mut Model,
    args: Vec<String>,
    operation_name: String,
) -> Option<Message> {
    model.popup = None;

    if model.pty_state.is_some() {
        model.popup = Some(PopupContent::Error {
            message: "A command is already in progress".to_string(),
        });
        return None;
    }

    let repo_path = &model.workdir;

    // Spawn command in background thread with PTY
    let (result_rx, ui_channels) =
        spawn_git_with_pty(repo_path.to_path_buf(), args, CredentialStrategy::Prompt);

    // Store PTY state for main loop to monitor
    if let Some(ui_channels) = ui_channels {
        model.pty_state = Some(PtyState::new(
            result_rx,
            ui_channels.request_rx,
            ui_channels.response_tx,
            operation_name,
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
