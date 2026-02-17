use crate::{
    git::push::get_upstream_branch,
    model::{
        Model,
        popup::{PopupContent, PopupContentCommand, PullPopupState},
    },
    msg::Message,
};

pub fn update(model: &mut Model) -> Option<Message> {
    // Get the upstream branch if set
    let upstream = get_upstream_branch(&model.workdir).ok().flatten();

    let state = PullPopupState { upstream };

    model.popup = Some(PopupContent::Command(PopupContentCommand::Pull(state)));
    None
}
