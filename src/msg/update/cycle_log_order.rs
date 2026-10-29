use crate::{
    model::{Model, arguments::CommitOrder},
    msg::Message,
};

/// Cycle the `-o` commit ordering in place: none → topo → author-date → date →
/// none. The order lives in `model.arguments`, so the log popup stays open and
/// re-renders with the new selection.
pub fn update(model: &mut Model) -> Option<Message> {
    if let Some(order) = model.arguments.as_mut().and_then(|a| a.log_order_mut()) {
        *order = CommitOrder::next(*order);
    }
    model.arg_mode = false;
    None
}
