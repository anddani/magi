use std::collections::HashSet;

use crate::model::{Line, SectionType};

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
