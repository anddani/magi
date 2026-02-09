use crate::{
    model::{
        popup::{PopupContent, PopupContentCommand},
        Model,
    },
    msg::Message,
};

use super::push_helper::execute_push;

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
    // Get the remote and branch from popup state
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

    // Build the refspec for setting upstream
    let refspec = format!("HEAD:{}", branch);

    // Build extra arguments for setting upstream
    let extra_args = vec![
        "--set-upstream".to_string(),
        remote.clone(),
        refspec,
    ];

    let operation_name = format!("Push to {}/{}", remote, branch);

    execute_push(model, extra_args, operation_name)
}
