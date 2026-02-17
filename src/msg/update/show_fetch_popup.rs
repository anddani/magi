use crate::{
    git::push::get_upstream_branch,
    model::{
        Model,
        popup::{FetchPopupState, PopupContent, PopupContentCommand},
    },
    msg::Message,
};

pub fn update(model: &mut Model) -> Option<Message> {
    // Get the upstream branch if set
    let upstream = get_upstream_branch(&model.workdir).ok().flatten();

    let state = FetchPopupState { upstream };

    model.popup = Some(PopupContent::Command(PopupContentCommand::Fetch(state)));
    None
}
