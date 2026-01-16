use std::collections::HashSet;

use crate::{
    model::{Line, SectionType},
    msg::Message,
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
    match msg {
        Message::Commit => true,

        Message::Quit
        | Message::Refresh
        | Message::MoveUp
        | Message::MoveDown
        | Message::HalfPageUp
        | Message::HalfPageDown
        | Message::ScrollLineDown
        | Message::ScrollLineUp
        | Message::ToggleSection
        | Message::StageAllModified
        | Message::UnstageAll
        | Message::DismissDialog
        | Message::EnterVisualMode
        | Message::ExitVisualMode
        | Message::ShowHelp => false,
    }
}
