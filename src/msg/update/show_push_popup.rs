use crate::{
    git::push::get_upstream_branch,
    model::{
        Model,
        popup::{PopupContent, PopupContentCommand, PushPopupState},
    },
    msg::Message,
};

pub fn update(model: &mut Model) -> Option<Message> {
    // Get the upstream branch if set
    let upstream = get_upstream_branch(&model.workdir).ok().flatten();

    let state = PushPopupState { upstream };

    model.popup = Some(PopupContent::Command(PopupContentCommand::Push(state)));
    None
}
