use crate::{
    git::{checkout::get_local_branches, open_pr::has_upstream},
    model::{
        Model,
        popup::{PopupContent, PopupContentCommand, SelectPopupState},
        select_popup::SelectContext,
    },
    msg::Message,
};

pub fn update(model: &mut Model) -> Option<Message> {
    model.popup = None;

    let repo_path = match model.git_info.repository.workdir() {
        Some(path) => path.to_path_buf(),
        None => {
            model.popup = Some(PopupContent::Error {
                message: "Could not determine repository path".to_string(),
            });
            return None;
        }
    };

    let current_branch = match model.git_info.current_branch() {
        Some(branch) => branch,
        None => {
            model.popup = Some(PopupContent::Error {
                message: "Could not determine current branch (detached HEAD?)".to_string(),
            });
            return None;
        }
    };

    if !has_upstream(&repo_path, &current_branch) {
        model.popup = Some(PopupContent::Error {
            message: format!(
                "Branch '{}' has no upstream. Push it first.",
                current_branch
            ),
        });
        return None;
    }

    let branches: Vec<String> = get_local_branches(&model.git_info.repository)
        .into_iter()
        .filter(|b| b != &current_branch)
        .collect();

    if branches.is_empty() {
        model.popup = Some(PopupContent::Error {
            message: "No other local branches found".to_string(),
        });
        return None;
    }

    model.select_context = Some(SelectContext::OpenPrTarget);

    let state = SelectPopupState::new("Open PR to".to_string(), branches);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Select(state)));

    None
}
