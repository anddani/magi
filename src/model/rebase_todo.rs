use crate::git::rebase::{RebaseAction, RebaseTodoEntry};

/// State for the interactive rebase todo editor (ViewMode::RebaseTodo).
///
/// Holds the todo entries being edited plus an undo stack. The state
/// guarantees that a fold action (squash/fixup) never ends up as the first
/// entry, since git rejects such a todo list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RebaseTodoState {
    /// The selected base commit (the oldest commit included in the rebase)
    pub base: String,
    /// Whether the base commit has a parent; false means `rebase --root`
    pub base_has_parent: bool,
    /// The todo entries, oldest commit first (same order as the todo file)
    pub entries: Vec<RebaseTodoEntry>,
    /// Snapshots of `entries` for undo, most recent last
    undo_stack: Vec<Vec<RebaseTodoEntry>>,
}

impl RebaseTodoState {
    pub fn new(base: String, base_has_parent: bool, entries: Vec<RebaseTodoEntry>) -> Self {
        Self {
            base,
            base_has_parent,
            entries,
            undo_stack: Vec::new(),
        }
    }

    /// Sets the action of the entry at `index`.
    /// Returns false when the change is rejected (fold action on the first
    /// entry) or `index` is out of range.
    pub fn set_action(&mut self, index: usize, action: RebaseAction) -> bool {
        if index >= self.entries.len() || (index == 0 && action.is_fold()) {
            return false;
        }
        if self.entries[index].action == action {
            return true;
        }
        self.snapshot();
        self.entries[index].action = action;
        true
    }

    /// Swaps the entry at `index` with the one above it.
    /// Returns false at the top boundary, or when the swap would put a fold
    /// action first.
    pub fn move_entry_up(&mut self, index: usize) -> bool {
        if index == 0 || index >= self.entries.len() {
            return false;
        }
        if index == 1 && self.entries[index].action.is_fold() {
            return false;
        }
        self.snapshot();
        self.entries.swap(index, index - 1);
        true
    }

    /// Swaps the entry at `index` with the one below it.
    /// Returns false at the bottom boundary, or when the swap would put a
    /// fold action first.
    pub fn move_entry_down(&mut self, index: usize) -> bool {
        if index + 1 >= self.entries.len() {
            return false;
        }
        if index == 0 && self.entries[index + 1].action.is_fold() {
            return false;
        }
        self.snapshot();
        self.entries.swap(index, index + 1);
        true
    }

    /// Restores the most recent snapshot. Returns false when there is
    /// nothing to undo.
    pub fn undo(&mut self) -> bool {
        match self.undo_stack.pop() {
            Some(entries) => {
                self.entries = entries;
                true
            }
            None => false,
        }
    }

    fn snapshot(&mut self) {
        self.undo_stack.push(self.entries.clone());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(hash: &str, message: &str) -> RebaseTodoEntry {
        RebaseTodoEntry {
            action: RebaseAction::Pick,
            hash: hash.to_string(),
            message: message.to_string(),
        }
    }

    fn state_with_entries(count: usize) -> RebaseTodoState {
        let entries = (0..count)
            .map(|i| entry(&format!("hash{i}"), &format!("Commit {i}")))
            .collect();
        RebaseTodoState::new("base".to_string(), true, entries)
    }

    #[test]
    fn test_set_action_changes_entry() {
        let mut state = state_with_entries(3);
        assert!(state.set_action(1, RebaseAction::Squash));
        assert_eq!(state.entries[1].action, RebaseAction::Squash);
        assert_eq!(state.entries[0].action, RebaseAction::Pick);
    }

    #[test]
    fn test_set_action_rejects_fold_on_first_entry() {
        let mut state = state_with_entries(3);
        assert!(!state.set_action(0, RebaseAction::Squash));
        assert!(!state.set_action(0, RebaseAction::Fixup));
        assert_eq!(state.entries[0].action, RebaseAction::Pick);
    }

    #[test]
    fn test_set_action_allows_non_fold_on_first_entry() {
        let mut state = state_with_entries(3);
        assert!(state.set_action(0, RebaseAction::Reword));
        assert_eq!(state.entries[0].action, RebaseAction::Reword);
    }

    #[test]
    fn test_set_action_out_of_range() {
        let mut state = state_with_entries(2);
        assert!(!state.set_action(2, RebaseAction::Drop));
    }

    #[test]
    fn test_move_entry_up_swaps() {
        let mut state = state_with_entries(3);
        assert!(state.move_entry_up(2));
        assert_eq!(state.entries[1].hash, "hash2");
        assert_eq!(state.entries[2].hash, "hash1");
    }

    #[test]
    fn test_move_entry_up_at_top_boundary() {
        let mut state = state_with_entries(3);
        assert!(!state.move_entry_up(0));
    }

    #[test]
    fn test_move_entry_up_rejects_fold_moving_to_top() {
        let mut state = state_with_entries(3);
        state.set_action(1, RebaseAction::Fixup);
        assert!(!state.move_entry_up(1));
        assert_eq!(state.entries[1].action, RebaseAction::Fixup);
        assert_eq!(state.entries[0].hash, "hash0");
    }

    #[test]
    fn test_move_entry_down_swaps() {
        let mut state = state_with_entries(3);
        assert!(state.move_entry_down(0));
        assert_eq!(state.entries[0].hash, "hash1");
        assert_eq!(state.entries[1].hash, "hash0");
    }

    #[test]
    fn test_move_entry_down_at_bottom_boundary() {
        let mut state = state_with_entries(3);
        assert!(!state.move_entry_down(2));
    }

    #[test]
    fn test_move_entry_down_rejects_fold_moving_to_top() {
        let mut state = state_with_entries(3);
        state.set_action(1, RebaseAction::Squash);
        // Moving the first entry down would make the squash entry first
        assert!(!state.move_entry_down(0));
        assert_eq!(state.entries[0].hash, "hash0");
    }

    #[test]
    fn test_undo_restores_previous_entries() {
        let mut state = state_with_entries(3);
        state.set_action(1, RebaseAction::Drop);
        state.move_entry_up(2);

        assert!(state.undo());
        assert_eq!(state.entries[1].hash, "hash1");
        assert_eq!(state.entries[1].action, RebaseAction::Drop);

        assert!(state.undo());
        assert_eq!(state.entries[1].action, RebaseAction::Pick);

        assert!(!state.undo());
    }

    #[test]
    fn test_set_action_same_action_does_not_push_undo() {
        let mut state = state_with_entries(2);
        assert!(state.set_action(0, RebaseAction::Pick));
        assert!(!state.undo());
    }
}
