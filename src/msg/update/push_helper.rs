use std::collections::HashSet;

use crate::{
    git::{credential::CredentialStrategy, pty_command::spawn_git_with_pty},
    model::{
        arguments::{Arguments::PushArguments, PushArgument},
        popup::PopupContent,
        Model, PtyState,
    },
    msg::Message,
};

/// Execute a push command with the given extra arguments.
///
/// This handles the common push logic:
/// - Checking for existing PTY state
/// - Getting the repository path
/// - Building args with push arguments from model
/// - Spawning the PTY command
/// - Setting up PtyState or showing error popup
///
/// # Arguments
/// * `model` - The application model
/// * `extra_args` - Additional arguments to pass to git push (e.g., `["--set-upstream", "origin", "HEAD:main"]`)
/// * `operation_name` - Name to display for this operation (e.g., "Push" or "Push to origin/main")
pub fn execute_push(
    model: &mut Model,
    extra_args: Vec<String>,
    operation_name: String,
) -> Option<Message> {
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

    // Add push arguments from model (e.g., --force-with-lease, --force)
    let arguments: HashSet<PushArgument> =
        if let Some(PushArguments(arguments)) = model.arguments.take() {
            arguments
        } else {
            HashSet::new()
        };
    for argument in arguments {
        args.push(argument.into());
    }

    // Add extra arguments
    args.extend(extra_args);

    // Spawn push command in background thread with PTY
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
