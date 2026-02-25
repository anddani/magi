use crate::{model::Model, msg::Message};

use super::pty_helper::execute_pty_command;

pub fn update(model: &mut Model, message: String) -> Option<Message> {
    let mut args = vec!["stash".to_string(), "push".to_string()];
    if !message.is_empty() {
        args.extend(["-m".to_string(), message]);
    }
    execute_pty_command(model, args, "Stash".to_string())
}
