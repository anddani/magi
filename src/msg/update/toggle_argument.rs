use std::collections::HashSet;

use crate::{
    model::{
        arguments::{Arguments::PushArguments, PushArgument},
        Model,
    },
    msg::Message,
};

pub fn update(model: &mut Model, argument: PushArgument) -> Option<Message> {
    match &mut model.arguments {
        Some(PushArguments(ref mut set)) => {
            // Toggle: remove if present, add if absent
            if !set.remove(&argument) {
                set.insert(argument);
            }
        }
        _ => {
            // No arguments or different variant: create new PushArguments with this argument
            let mut set = HashSet::new();
            set.insert(argument);
            model.arguments = Some(PushArguments(set));
        }
    }
    Some(Message::ExitArgMode)
}
