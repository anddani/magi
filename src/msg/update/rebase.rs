use crate::{
    model::Model,
    msg::{Message, RebaseCommand, update::pty_helper::execute_pty_command},
};

pub fn update(model: &mut Model, rebase_command: RebaseCommand) -> Option<Message> {
    match rebase_command {
        RebaseCommand::Elsewhere(target) => elsewhere(model, target),
    }
}

fn elsewhere(model: &mut Model, target: String) -> Option<Message> {
    let args = vec!["rebase".to_string(), target];
    execute_pty_command(model, args, "Rebase".to_string())
}
