use crate::{
    model::Model,
    msg::{MergeCommand, Message, update::pty_helper::execute_pty_command},
};

pub fn update(model: &mut Model, cmd: MergeCommand) -> Option<Message> {
    match cmd {
        MergeCommand::Branch(branch) => merge_branch(model, branch),
        MergeCommand::Continue => continue_merge(model),
        MergeCommand::Abort => abort_merge(model),
    }
}

fn merge_branch(model: &mut Model, branch: String) -> Option<Message> {
    model.popup = None;
    execute_pty_command(
        model,
        vec!["merge".to_string(), branch],
        "Merge".to_string(),
    )
}

fn continue_merge(model: &mut Model) -> Option<Message> {
    model.popup = None;
    execute_pty_command(
        model,
        vec!["merge".to_string(), "--continue".to_string()],
        "Merge".to_string(),
    )
}

fn abort_merge(model: &mut Model) -> Option<Message> {
    model.popup = None;
    execute_pty_command(
        model,
        vec!["merge".to_string(), "--abort".to_string()],
        "Merge".to_string(),
    )
}
