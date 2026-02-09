
use crate::{
    model::{
        arguments::Arguments::FetchArguments,
        Model,
    },
    msg::Message,
};

use super::pty_helper::execute_pty_command;

pub fn update(model: &mut Model) -> Option<Message> {
    let mut args = vec!["fetch".to_string(), "-v".to_string(), "--all".to_string()];

    if let Some(FetchArguments(arguments)) = model.arguments.take() {
        args.extend(arguments.into_iter().map(|a| a.flag().to_string()));
    }

    execute_pty_command(model, args, "Fetch all".to_string())
}
