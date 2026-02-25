use crate::{model::Model, msg::Message};

use super::pty_helper::execute_pty_command;

pub fn update(model: &mut Model, stash_ref: String) -> Option<Message> {
    let args = vec!["stash".to_string(), "apply".to_string(), stash_ref];
    execute_pty_command(model, args, "Stash apply".to_string())
}
