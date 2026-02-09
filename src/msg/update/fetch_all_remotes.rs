use std::collections::HashSet;

use crate::{
    model::{
        arguments::{Arguments::FetchArguments, FetchArgument},
        Model,
    },
    msg::Message,
};

use super::pty_helper::execute_pty_command;

pub fn update(model: &mut Model) -> Option<Message> {
    let mut args = vec!["fetch".to_string(), "-v".to_string(), "--all".to_string()];

    let arguments: HashSet<FetchArgument> =
        if let Some(FetchArguments(arguments)) = model.arguments.take() {
            arguments
        } else {
            HashSet::new()
        };
    for argument in arguments {
        args.push(argument.flag().to_string());
    }

    execute_pty_command(model, args, "Fetch all".to_string())
}
