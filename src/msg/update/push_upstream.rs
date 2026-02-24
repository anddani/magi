use crate::{
    git::push::{get_current_branch, get_upstream_branch, parse_remote_branch},
    model::Model,
    msg::Message,
};

use super::push_helper::execute_push;

pub fn update(model: &mut Model) -> Option<Message> {
    // When the upstream branch name differs from the local branch name, git push
    // refuses to run without an explicit refspec. Detect this and supply one.
    if let (Ok(Some(upstream)), Ok(Some(local))) = (
        get_upstream_branch(&model.workdir),
        get_current_branch(&model.workdir),
    ) {
        let (remote, remote_branch) = parse_remote_branch(&upstream);
        if !remote_branch.is_empty() && remote_branch != local {
            let refspec = format!("HEAD:{}", remote_branch);
            return execute_push(model, vec![remote, refspec], "Push".to_string());
        }
    }

    execute_push(model, vec![], "Push".to_string())
}
