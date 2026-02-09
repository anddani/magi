use crate::{model::Model, msg::Message};

use super::push_helper::execute_push;

pub fn update(model: &mut Model) -> Option<Message> {
    execute_push(model, vec![], "Push".to_string())
}
