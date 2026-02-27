use crate::{
    model::Model,
    msg::{Message, RebaseCommand, update::pty_helper::execute_pty_command},
};

pub fn update(model: &mut Model, rebase_command: RebaseCommand) -> Option<Message> {
    match rebase_command {
        RebaseCommand::Elsewhere(target) => elsewhere(model, target),
        RebaseCommand::Continue => continue_rebase(model),
        RebaseCommand::Skip => skip_rebase(model),
        RebaseCommand::Abort => abort_rebase(model),
    }
}

fn elsewhere(model: &mut Model, target: String) -> Option<Message> {
    let args = vec!["rebase".to_string(), target];
    execute_pty_command(model, args, "Rebase".to_string())
}

fn continue_rebase(model: &mut Model) -> Option<Message> {
    execute_pty_command(
        model,
        vec!["rebase".to_string(), "--continue".to_string()],
        "Rebase".to_string(),
    )
}

fn skip_rebase(model: &mut Model) -> Option<Message> {
    execute_pty_command(
        model,
        vec!["rebase".to_string(), "--skip".to_string()],
        "Rebase".to_string(),
    )
}

fn abort_rebase(model: &mut Model) -> Option<Message> {
    execute_pty_command(
        model,
        vec!["rebase".to_string(), "--abort".to_string()],
        "Rebase".to_string(),
    )
}
