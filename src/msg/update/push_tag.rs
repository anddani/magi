use crate::{model::Model, msg::Message};

use super::push_helper::execute_push;

pub fn update(model: &mut Model, tag: String) -> Option<Message> {
    execute_push(
        model,
        vec!["origin".to_string(), tag.clone()],
        format!("Push tag {}", tag),
    )
}
