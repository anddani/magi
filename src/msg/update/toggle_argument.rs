use std::collections::HashSet;
use std::hash::Hash;

use crate::{
    model::{
        Model,
        arguments::{Argument, Arguments},
    },
    msg::Message,
};

pub fn update(model: &mut Model, argument: Argument) -> Option<Message> {
    match argument {
        Argument::Commit(arg) => toggle_set(
            &mut model.arguments,
            arg,
            |a| a.commit_mut(),
            Arguments::CommitArguments,
        ),
        Argument::Push(arg) => toggle_set(
            &mut model.arguments,
            arg,
            |a| a.push_mut(),
            Arguments::PushArguments,
        ),
        Argument::Fetch(arg) => toggle_set(
            &mut model.arguments,
            arg,
            |a| a.fetch_mut(),
            Arguments::FetchArguments,
        ),
        Argument::Stash(arg) => toggle_set(
            &mut model.arguments,
            arg,
            |a| a.stash_mut(),
            Arguments::StashArguments,
        ),
        Argument::Pull(arg) => toggle_set(
            &mut model.arguments,
            arg,
            |a| a.pull_mut(),
            Arguments::PullArguments,
        ),
    }
    model.arg_mode = false;
    None
}

fn toggle_set<A: Eq + Hash>(
    current: &mut Option<Arguments>,
    arg: A,
    get_mut: impl FnOnce(&mut Arguments) -> Option<&mut HashSet<A>>,
    make: impl FnOnce(HashSet<A>) -> Arguments,
) {
    if let Some(set) = current.as_mut().and_then(get_mut) {
        if !set.remove(&arg) {
            set.insert(arg);
        }
    } else {
        *current = Some(make([arg].into_iter().collect()));
    }
}
