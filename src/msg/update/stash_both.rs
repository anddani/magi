use crate::{
    model::{Model, arguments::Arguments::StashArguments},
    msg::Message,
};

use super::pty_helper::execute_pty_command;

pub fn update(model: &mut Model, message: String) -> Option<Message> {
    let mut args = vec!["stash".to_string(), "push".to_string()];
    if !message.is_empty() {
        args.extend(["-m".to_string(), message]);
    }

    let flags = if let Some(StashArguments(arguments)) = model.arguments.take() {
        arguments
            .into_iter()
            .map(|a| a.flag().to_string())
            .collect()
    } else {
        vec![]
    };
    args.extend(flags);

    execute_pty_command(model, args, "Stash".to_string())
}
