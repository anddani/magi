use std::collections::HashSet;

use crate::{
    model::{Line, SectionType},
    msg::{Message, RebaseCommand, RevertCommand},
};

/// Count visible lines between two raw line indices (exclusive of end).
/// Used to calculate scroll offsets in terms of visible lines.
pub fn visible_lines_between(
    lines: &[Line],
    start: usize,
    end: usize,
    collapsed: &HashSet<SectionType>,
) -> usize {
    lines
        .iter()
        .skip(start)
        .take(end.saturating_sub(start))
        .filter(|line| !line.is_hidden(collapsed))
        .count()
}

/// Returns true if [`Message`] requires to pause Ratatui rendering
pub fn is_external_command(msg: &Message) -> bool {
    matches!(
        msg,
        Message::Commit
            | Message::Amend(_)
            | Message::FixupCommit(_, _)
            | Message::Revert(RevertCommand::Continue)
            | Message::Rebase(RebaseCommand::Continue)
    )
}
