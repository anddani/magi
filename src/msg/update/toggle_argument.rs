use std::collections::HashSet;

use crate::{
    model::{
        arguments::{Argument, Arguments, FetchArgument, PushArgument},
        Model,
    },
    msg::Message,
};

pub fn update(model: &mut Model, argument: Argument) -> Option<Message> {
    match argument {
        Argument::Push(push_arg) => toggle_push_argument(model, push_arg),
        Argument::Fetch(fetch_arg) => toggle_fetch_argument(model, fetch_arg),
        Argument::Pull(_pull_arg) => {
            // TODO: Implement pull arguments when needed
        }
    }
    // Exit arg mode after toggling
    model.arg_mode = false;
    None
}

fn toggle_push_argument(model: &mut Model, argument: PushArgument) {
    match &mut model.arguments {
        Some(Arguments::PushArguments(ref mut set)) => {
            if !set.remove(&argument) {
                set.insert(argument);
            }
        }
        _ => {
            let mut set = HashSet::new();
            set.insert(argument);
            model.arguments = Some(Arguments::PushArguments(set));
        }
    }
}

fn toggle_fetch_argument(model: &mut Model, argument: FetchArgument) {
    match &mut model.arguments {
        Some(Arguments::FetchArguments(ref mut set)) => {
            if !set.remove(&argument) {
                set.insert(argument);
            }
        }
        _ => {
            let mut set = HashSet::new();
            set.insert(argument);
            model.arguments = Some(Arguments::FetchArguments(set));
        }
    }
}
