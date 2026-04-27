use crate::{
    model::Model,
    msg::{ApplyCommand, Message, update::pty_helper::execute_pty_command},
};

pub fn update(model: &mut Model, cmd: ApplyCommand) -> Option<Message> {
    match cmd {
        ApplyCommand::Pick(hashes) => pick(model, hashes),
        ApplyCommand::Apply(hashes) => apply_no_commit(model, hashes),
        ApplyCommand::Squash(hash) => squash(model, hash),
        ApplyCommand::Continue => continue_apply(model),
        ApplyCommand::Skip => skip_apply(model),
        ApplyCommand::Abort => abort_apply(model),
    }
}

fn pick(model: &mut Model, hashes: Vec<String>) -> Option<Message> {
    if hashes.is_empty() {
        return None;
    }
    let mut args = vec!["cherry-pick".to_string()];
    args.extend(hashes);
    execute_pty_command(model, args, "Apply".to_string())
}

fn apply_no_commit(model: &mut Model, hashes: Vec<String>) -> Option<Message> {
    if hashes.is_empty() {
        return None;
    }
    let mut args = vec!["cherry-pick".to_string(), "--no-commit".to_string()];
    args.extend(hashes);
    execute_pty_command(model, args, "Apply".to_string())
}

fn squash(model: &mut Model, hash: String) -> Option<Message> {
    execute_pty_command(
        model,
        vec!["merge".to_string(), "--squash".to_string(), hash],
        "Apply".to_string(),
    )
}

fn continue_apply(model: &mut Model) -> Option<Message> {
    execute_pty_command(
        model,
        vec!["cherry-pick".to_string(), "--continue".to_string()],
        "Apply".to_string(),
    )
}

fn skip_apply(model: &mut Model) -> Option<Message> {
    execute_pty_command(
        model,
        vec!["cherry-pick".to_string(), "--skip".to_string()],
        "Apply".to_string(),
    )
}

fn abort_apply(model: &mut Model) -> Option<Message> {
    execute_pty_command(
        model,
        vec!["cherry-pick".to_string(), "--abort".to_string()],
        "Apply".to_string(),
    )
}
