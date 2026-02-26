use crate::{
    model::Model,
    msg::{Message, StashCommand},
};

pub fn update(model: &mut Model, stash_ref: String) -> Option<Message> {
    // Clear the popup
    model.popup = None;

    // Return message to drop the stash
    Some(Message::Stash(StashCommand::Drop(stash_ref)))
}
