use std::collections::HashSet;

use crate::{
    model::{
        arguments::{Arguments::PushArguments, PushArgument},
        Model,
    },
    msg::Message,
};

use super::pty_helper::execute_pty_command;

/// Execute a push command with the given extra arguments.
///
/// This handles the common push logic:
/// - Building args with push arguments from model
/// - Calling the generic PTY command executor
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
        args.push(argument.flag().to_string());
    }

    // Add extra arguments
    args.extend(extra_args);

    execute_pty_command(model, args, operation_name)
}
