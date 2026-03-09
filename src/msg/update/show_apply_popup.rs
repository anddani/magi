use crate::{
    git::cherry_pick::cherry_pick_in_progress,
    model::{
        Model,
        popup::{ApplyPopupState, PopupContent, PopupContentCommand},
    },
    msg::Message,
};

pub fn update(model: &mut Model) -> Option<Message> {
    let in_progress = cherry_pick_in_progress(&model.workdir);
    let state = ApplyPopupState { in_progress };
    model.popup = Some(PopupContent::Command(PopupContentCommand::Apply(state)));
    None
}
