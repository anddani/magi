use crate::{
    git::commit::run_revise_commit,
    i18n,
    model::{Model, popup::PopupContent},
    msg::Message,
};

pub fn update(model: &mut Model, commit_hash: String) -> Option<Message> {
    let repo_path = match model.git_info.repository.workdir() {
        Some(path) => path,
        None => {
            model.popup = Some(PopupContent::Error {
                message: i18n::t().err_no_working_directory.to_string(),
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
                message: format!("{}: {}", i18n::t().err_revise_commit_failed, err),
            });
            None
        }
    }
}
