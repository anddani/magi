use crate::{
    git::checkout::{BranchEntry, get_all_branches},
    model::{
        Model,
        popup::{PopupContent, PopupContentCommand, SelectPopupState},
        select_popup::SelectContext,
    },
    msg::Message,
};

pub fn update(model: &mut Model, remote: String) -> Option<Message> {
    let prefix = format!("{}/", remote);

    let branches: Vec<String> = get_all_branches(&model.git_info.repository)
        .into_iter()
        .filter_map(|b| match b {
            BranchEntry::Remote(name) if name.starts_with(&prefix) => Some(name),
            _ => None,
        })
        .collect();

    if branches.is_empty() {
        model.popup = Some(PopupContent::Error {
            message: format!("No branches found for remote '{}'", remote),
        });
        return None;
    }

    model.select_context = Some(SelectContext::FetchAnotherBranch);

    let state = SelectPopupState::new(format!("Fetch branch from {}", remote), branches);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Select(state)));

    None
}
