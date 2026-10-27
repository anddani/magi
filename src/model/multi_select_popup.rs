use std::collections::HashSet;

use crate::model::InputField;

/// What to do when a multi-select popup is confirmed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OnMultiSelect {
    /// Stash the selected files with `git stash push <flags> -- <files>`.
    /// The flags are snapshotted from the stash-push arguments when the
    /// picker is opened so they survive the popup swap.
    StashPushFiles { flags: Vec<String> },
}

/// State for a multi-select popup (fuzzy finder that toggles several items).
///
/// Mirrors [`super::select_popup::SelectPopupState`] but lets the user check
/// any number of items (SPACE toggles the item under the cursor) before
/// confirming with RET.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MultiSelectPopupState {
    /// Display title
    pub title: String,
    /// All available options (unfiltered)
    pub all_options: Vec<String>,
    /// Indices into `all_options` that are currently checked
    pub checked: HashSet<usize>,
    /// Indices of options matching the current filter
    pub filtered_indices: Vec<usize>,
    /// Current filter text
    pub input: InputField,
    /// Currently highlighted position within `filtered_indices` (0-based)
    pub selected_index: usize,
    /// What to do when the user confirms the selection
    pub on_confirm: OnMultiSelect,
}

impl MultiSelectPopupState {
    /// Create a new multi-select popup with nothing checked.
    pub fn new(title: String, options: Vec<String>, on_confirm: OnMultiSelect) -> Self {
        let filtered_indices: Vec<usize> = (0..options.len()).collect();
        Self {
            title,
            all_options: options,
            checked: HashSet::new(),
            filtered_indices,
            input: InputField::new(),
            selected_index: 0,
            on_confirm,
        }
    }

    /// The `all_options` index currently under the cursor, if any.
    fn highlighted_option(&self) -> Option<usize> {
        self.filtered_indices.get(self.selected_index).copied()
    }

    /// Toggle the checked state of the item under the cursor.
    pub fn toggle_current(&mut self) {
        if let Some(idx) = self.highlighted_option()
            && !self.checked.remove(&idx)
        {
            self.checked.insert(idx);
        }
    }

    /// Whether the given `all_options` index is checked.
    pub fn is_checked(&self, option_index: usize) -> bool {
        self.checked.contains(&option_index)
    }

    /// The checked files, in `all_options` order.
    pub fn checked_items(&self) -> Vec<String> {
        self.all_options
            .iter()
            .enumerate()
            .filter(|(idx, _)| self.checked.contains(idx))
            .map(|(_, opt)| opt.clone())
            .collect()
    }

    /// Updates `filtered_indices` based on the current filter text
    /// (case-insensitive substring). Checked state is preserved because it is
    /// keyed on `all_options` indices, not filtered positions.
    pub fn update_filter(&mut self) {
        let query = self.input.as_str().to_lowercase();
        self.filtered_indices = self
            .all_options
            .iter()
            .enumerate()
            .filter(|(_, opt)| opt.to_lowercase().contains(&query))
            .map(|(idx, _)| idx)
            .collect();
        self.selected_index = 0;
    }

    /// Number of filtered hits
    pub fn filtered_count(&self) -> usize {
        self.filtered_indices.len()
    }

    /// Move the cursor up
    pub fn move_up(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }

    /// Move the cursor down
    pub fn move_down(&mut self) {
        if self.selected_index + 1 < self.filtered_indices.len() {
            self.selected_index += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn on_confirm() -> OnMultiSelect {
        OnMultiSelect::StashPushFiles { flags: vec![] }
    }

    fn state() -> MultiSelectPopupState {
        MultiSelectPopupState::new(
            "Stash push".to_string(),
            vec!["a.rs".to_string(), "b.rs".to_string(), "c.rs".to_string()],
            on_confirm(),
        )
    }

    #[test]
    fn test_new_checks_nothing() {
        let s = state();
        assert!(s.checked_items().is_empty());
        assert_eq!(s.filtered_indices, vec![0, 1, 2]);
    }

    #[test]
    fn test_toggle_current_checks_and_unchecks() {
        let mut s = state();
        s.toggle_current();
        assert_eq!(s.checked_items(), vec!["a.rs".to_string()]);
        s.toggle_current();
        assert!(s.checked_items().is_empty());
    }

    #[test]
    fn test_checked_items_preserved_across_filter() {
        let mut s = state();
        // check c.rs (index 2)
        s.selected_index = 2;
        s.toggle_current();
        assert_eq!(s.checked_items(), vec!["c.rs".to_string()]);

        // filter to only "b", selection resets but c.rs stays checked
        s.input = InputField::from_text("b");
        s.update_filter();
        assert_eq!(s.filtered_indices, vec![1]);
        assert_eq!(s.selected_index, 0);
        assert_eq!(s.checked_items(), vec!["c.rs".to_string()]);
    }

    #[test]
    fn test_checked_items_returns_all_options_order() {
        let mut s = state();
        s.selected_index = 2;
        s.toggle_current();
        s.selected_index = 0;
        s.toggle_current();
        assert_eq!(
            s.checked_items(),
            vec!["a.rs".to_string(), "c.rs".to_string()]
        );
    }

    #[test]
    fn test_move_up_and_down_bounds() {
        let mut s = state();
        assert_eq!(s.selected_index, 0);
        s.move_up();
        assert_eq!(s.selected_index, 0);
        s.move_down();
        s.move_down();
        s.move_down();
        assert_eq!(s.selected_index, 2);
    }
}
