use crate::{
    git::push::get_upstream_branch,
    model::{
        Model,
        popup::{FetchPopupState, PopupContent, PopupContentCommand},
    },
    msg::Message,
};

pub fn update(model: &mut Model) -> Option<Message> {
    let Some(repo_path) = model.git_info.repository.workdir() else {
        model.popup = Some(PopupContent::Error {
            message: "Repository working directory not found".to_string(),
        });
        return None;
    };

    // Get the upstream branch if set
    let upstream = get_upstream_branch(repo_path).ok().flatten();

    let state = FetchPopupState { upstream };

    model.popup = Some(PopupContent::Command(PopupContentCommand::Fetch(state)));
    None
}
