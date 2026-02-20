use crate::{
    git::commit::run_fixup_commit,
    model::{Model, popup::PopupContent},
    msg::Message,
};

pub fn update(model: &mut Model, commit_ref: String) -> Option<Message> {
    let repo_path = match model.git_info.repository.workdir() {
        Some(path) => path,
        None => {
            model.popup = Some(PopupContent::Error {
                message: "No working directory found".to_string(),
            });
            return None;
        }
    };

    // Extract the commit hash from the selection
    // Format is "hash - message", so we take the first part
    let commit_hash = commit_ref
        .split(" - ")
        .next()
        .unwrap_or(&commit_ref)
        .trim()
        .to_string();

    match run_fixup_commit(repo_path, commit_hash) {
        Ok(result) => {
            if result.success {
                Some(Message::Refresh)
            } else {
                model.popup = Some(PopupContent::Error {
                    message: result.message,
                });
                None
            }
        }
        Err(err) => {
            model.popup = Some(PopupContent::Error {
                message: format!("Fixup commit failed: {}", err),
            });
            None
        }
    }
}
