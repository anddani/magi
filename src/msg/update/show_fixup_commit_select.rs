use crate::{
    git::commit::get_recent_commits_for_fixup,
    model::{
        Model,
        popup::{PopupContent, PopupContentCommand, SelectContext, SelectPopupState},
    },
    msg::Message,
};

pub fn update(model: &mut Model) -> Option<Message> {
    let repo_path = model.git_info.repository.workdir()?;

    match get_recent_commits_for_fixup(repo_path) {
        Ok(commits) => {
            if commits.is_empty() {
                model.popup = Some(PopupContent::Error {
                    message: "No commits found in current branch".to_string(),
                });
                None
            } else {
                let state = SelectPopupState::new("Fixup commit".to_string(), commits);
                model.popup = Some(PopupContent::Command(PopupContentCommand::Select(state)));
                model.select_context = Some(SelectContext::FixupCommit);
                None
            }
        }
        Err(err) => {
            model.popup = Some(PopupContent::Error {
                message: format!("Failed to get recent commits: {}", err),
            });
            None
        }
    }
}
