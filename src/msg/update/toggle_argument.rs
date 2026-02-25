use std::collections::HashSet;

use crate::{
    model::{
        Model,
        arguments::{
            Argument, Arguments, CommitArgument, FetchArgument, PullArgument, PushArgument,
            StashArgument,
        },
    },
    msg::Message,
};

pub fn update(model: &mut Model, argument: Argument) -> Option<Message> {
    match argument {
        Argument::Commit(push_arg) => toggle_commit_argument(model, push_arg),
        Argument::Push(push_arg) => toggle_push_argument(model, push_arg),
        Argument::Fetch(fetch_arg) => toggle_fetch_argument(model, fetch_arg),
        Argument::Stash(fetch_arg) => toggle_stash_argument(model, fetch_arg),
        Argument::Pull(pull_arg) => toggle_pull_argument(model, pull_arg),
    }
    // Exit arg mode after toggling
    model.arg_mode = false;
    None
}

fn toggle_commit_argument(model: &mut Model, argument: CommitArgument) {
    match &mut model.arguments {
        Some(Arguments::CommitArguments(set)) => {
            if !set.remove(&argument) {
                set.insert(argument);
            }
        }
        _ => {
            let mut set = HashSet::new();
            set.insert(argument);
            model.arguments = Some(Arguments::CommitArguments(set));
        }
    }
}

fn toggle_push_argument(model: &mut Model, argument: PushArgument) {
    match &mut model.arguments {
        Some(Arguments::PushArguments(set)) => {
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
        Some(Arguments::FetchArguments(set)) => {
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

fn toggle_stash_argument(model: &mut Model, argument: StashArgument) {
    match &mut model.arguments {
        Some(Arguments::StashArguments(set)) => {
            if !set.remove(&argument) {
                set.insert(argument);
            }
        }
        _ => {
            let mut set = HashSet::new();
            set.insert(argument);
            model.arguments = Some(Arguments::StashArguments(set));
        }
    }
}

fn toggle_pull_argument(model: &mut Model, argument: PullArgument) {
    match &mut model.arguments {
        Some(Arguments::PullArguments(set)) => {
            if !set.remove(&argument) {
                set.insert(argument);
            }
        }
        _ => {
            let mut set = HashSet::new();
            set.insert(argument);
            model.arguments = Some(Arguments::PullArguments(set));
        }
    }
}
