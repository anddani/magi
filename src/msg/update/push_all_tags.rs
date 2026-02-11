use crate::{model::Model, msg::Message};

use super::push_helper::execute_push;

pub fn update(model: &mut Model, remote: String) -> Option<Message> {
    execute_push(
        model,
        vec![remote, "--tags".to_string()],
        "Push tags".to_string(),
    )
}
