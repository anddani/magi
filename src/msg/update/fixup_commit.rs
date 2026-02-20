use crate::{
    git::commit::{run_fixup_commit, run_squash_commit},
    model::{Model, popup::PopupContent},
    msg::{FixupType, Message},
};

pub fn update(model: &mut Model, commit_ref: String, fixup_type: FixupType) -> Option<Message> {
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

    let result = match fixup_type {
        FixupType::Fixup => run_fixup_commit(repo_path, commit_hash),
        FixupType::Squash => run_squash_commit(repo_path, commit_hash),
    };

    match result {
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
            let operation = match fixup_type {
                FixupType::Fixup => "Fixup",
                FixupType::Squash => "Squash",
            };
            model.popup = Some(PopupContent::Error {
                message: format!("{} commit failed: {}", operation, err),
            });
            None
        }
    }
}
