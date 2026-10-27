use crate::{
    git::commit::{run_alter_commit, run_augment_commit, run_fixup_commit, run_squash_commit},
    i18n,
    model::{Model, popup::PopupContent},
    msg::{FixupType, Message},
};

pub fn update(model: &mut Model, commit_ref: String, fixup_type: FixupType) -> Option<Message> {
    let repo_path = match model.git_info.repository.workdir() {
        Some(path) => path,
        None => {
            model.popup = Some(PopupContent::Error {
                message: i18n::t().err_no_working_directory.to_string(),
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
        FixupType::Alter => run_alter_commit(repo_path, commit_hash),
        FixupType::Augment => run_augment_commit(repo_path, commit_hash),
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
                FixupType::Fixup => i18n::t().cmd_fixup,
                FixupType::Squash => i18n::t().cmd_squash,
                FixupType::Alter => i18n::t().cmd_alter,
                FixupType::Augment => i18n::t().cmd_augment,
            };
            model.popup = Some(PopupContent::Error {
                message: format!(
                    "{}: {}",
                    i18n::t().fmt1(i18n::t().err_commit_failed_fmt, operation),
                    err
                ),
            });
            None
        }
    }
}
