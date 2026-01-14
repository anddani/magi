use crate::{
    model::{Model, RunningState},
    msg::Message,
};

pub fn update(model: &mut Model) -> Option<Message> {
    model.running_state = RunningState::Done;
    None
}
