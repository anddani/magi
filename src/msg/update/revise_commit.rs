use crate::{
    git::commit::run_revise_commit,
    model::{Model, popup::PopupContent},
    msg::Message,
};

pub fn update(model: &mut Model, commit_hash: String) -> Option<Message> {
    let repo_path = match model.git_info.repository.workdir() {
        Some(path) => path,
        None => {
            model.popup = Some(PopupContent::Error {
                message: "No working directory found".to_string(),
            });
            return None;
        }
    };

    match run_revise_commit(repo_path, commit_hash) {
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
                message: format!("Revise commit failed: {}", err),
            });
            None
        }
    }
}
