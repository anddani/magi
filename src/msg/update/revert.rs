use crate::{
    model::Model,
    msg::{Message, RevertCommand, update::pty_helper::execute_pty_command},
};

pub fn update(model: &mut Model, cmd: RevertCommand) -> Option<Message> {
    match cmd {
        RevertCommand::Commits(hashes) => commits(model, hashes),
        RevertCommand::NoCommit(hashes) => no_commit(model, hashes),
        RevertCommand::Continue => continue_revert(model),
        RevertCommand::Skip => skip_revert(model),
        RevertCommand::Abort => abort_revert(model),
    }
}

fn commits(model: &mut Model, hashes: Vec<String>) -> Option<Message> {
    if hashes.is_empty() {
        return None;
    }
    let mut args = vec!["revert".to_string(), "--no-edit".to_string()];
    args.extend(hashes);
    execute_pty_command(model, args, "Revert".to_string())
}

fn no_commit(model: &mut Model, hashes: Vec<String>) -> Option<Message> {
    if hashes.is_empty() {
        return None;
    }
    let mut args = vec!["revert".to_string(), "--no-commit".to_string()];
    args.extend(hashes);
    execute_pty_command(model, args, "Revert".to_string())
}

fn continue_revert(model: &mut Model) -> Option<Message> {
    execute_pty_command(
        model,
        vec!["revert".to_string(), "--continue".to_string()],
        "Revert".to_string(),
    )
}

fn skip_revert(model: &mut Model) -> Option<Message> {
    execute_pty_command(
        model,
        vec!["revert".to_string(), "--skip".to_string()],
        "Revert".to_string(),
    )
}

fn abort_revert(model: &mut Model) -> Option<Message> {
    execute_pty_command(
        model,
        vec!["revert".to_string(), "--abort".to_string()],
        "Revert".to_string(),
    )
}
