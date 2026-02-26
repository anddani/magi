use crate::model::arguments::Arguments::StashArguments;
use crate::{
    model::Model,
    msg::{Message, StashCommand, StashType, update::pty_helper::execute_pty_command},
};

pub fn update(model: &mut Model, stash_command: StashCommand) -> Option<Message> {
    let extra_args = if let Some(StashArguments(arguments)) = model.arguments.take() {
        arguments
            .into_iter()
            .map(|a| a.flag().to_string())
            .collect()
    } else {
        vec![]
    };
    match stash_command {
        StashCommand::Push(stash_type, message) => push(model, stash_type, message, extra_args),
        StashCommand::Apply(stash_ref) => apply(model, stash_ref, extra_args),
        StashCommand::Pop(stash_ref) => pop(model, stash_ref, extra_args),
        StashCommand::Drop(stash_ref) => drop(model, stash_ref),
    }
}

fn push(
    model: &mut Model,
    stash_type: StashType,
    message: String,
    extra_args: Vec<String>,
) -> Option<Message> {
    let mut args = vec!["stash".to_string(), "push".to_string()];

    if let Some(flag) = stash_type.flag() {
        args.push(flag.to_string());
    }

    if !message.is_empty() {
        args.extend(["-m".to_string(), message]);
    }

    // extra_args only apply to StashType::Both (index/worktree ignore them)
    if stash_type == StashType::Both {
        args.extend(extra_args);
    }

    execute_pty_command(model, args, stash_type.pty_title().to_string())
}

fn apply(model: &mut Model, stash_ref: String, extra_args: Vec<String>) -> Option<Message> {
    let mut args = vec!["stash".to_string(), "apply".to_string(), stash_ref];
    args.extend(extra_args);
    execute_pty_command(model, args, "Stash apply".to_string())
}

fn pop(model: &mut Model, stash_ref: String, extra_args: Vec<String>) -> Option<Message> {
    let mut args = vec!["stash".to_string(), "pop".to_string(), stash_ref];
    args.extend(extra_args);
    execute_pty_command(model, args, "Stash pop".to_string())
}

fn drop(model: &mut Model, stash_ref: String) -> Option<Message> {
    let (args, title) = if &stash_ref == "all" {
        (
            vec!["stash".to_string(), "clear".to_string()],
            "Drop all stashes".to_string(),
        )
    } else {
        (
            vec!["stash".to_string(), "drop".to_string(), stash_ref],
            "Drop stash".to_string(),
        )
    };
    execute_pty_command(model, args, title)
}
