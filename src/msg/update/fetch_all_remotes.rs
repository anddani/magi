use crate::{model::Model, msg::Message};

use super::pty_helper::execute_pty_command;

pub fn update(model: &mut Model) -> Option<Message> {
    let args = vec![
        "fetch".to_string(),
        "-v".to_string(),
        "--all".to_string(),
    ];

    execute_pty_command(model, args, "Fetch all".to_string())
}
