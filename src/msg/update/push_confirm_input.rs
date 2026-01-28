use crate::{
    git::{credential::CredentialStrategy, pty_command::spawn_git_with_pty},
    model::{
        popup::{PopupContent, PopupContentCommand},
        Model, PtyState,
    },
    msg::Message,
};

/// Parse input into (remote, branch) tuple.
/// If input contains "/", split on first "/" to get remote and branch.
/// Otherwise, use the default remote and the input as the branch name.
fn parse_remote_branch(input: &str, default_remote: &str, local_branch: &str) -> (String, String) {
    let input = input.trim();
    if input.is_empty() {
        // Use defaults
        (default_remote.to_string(), local_branch.to_string())
    } else if let Some((remote, branch)) = input.split_once('/') {
        // User specified remote/branch
        (remote.to_string(), branch.to_string())
    } else {
        // User specified only branch, use default remote
        (default_remote.to_string(), input.to_string())
    }
}

pub fn update(model: &mut Model) -> Option<Message> {
    // Get the remote and branch to push to
    let (remote, branch) =
        if let Some(PopupContent::Command(PopupContentCommand::Push(ref state))) = model.popup {
            parse_remote_branch(
                &state.input_text,
                &state.default_remote,
                &state.local_branch,
            )
        } else {
            return None;
        };

    // Dismiss the popup
    model.popup = None;

    // Check if there's already a PTY command running
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

    // Build the refspec for setting upstream
    let refspec = format!("HEAD:{}", branch);

    // Spawn push command in background thread with PTY
    let (result_rx, ui_channels) = spawn_git_with_pty(
        repo_path.to_path_buf(),
        vec![
            "push".to_string(),
            "-v".to_string(),
            "--set-upstream".to_string(),
            remote.clone(),
            refspec,
        ],
        CredentialStrategy::Prompt,
    );

    // Store PTY state for main loop to monitor
    if let Some(ui_channels) = ui_channels {
        model.pty_state = Some(PtyState::new(
            result_rx,
            ui_channels.request_rx,
            ui_channels.response_tx,
            format!("Push to {}/{}", remote, branch),
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
