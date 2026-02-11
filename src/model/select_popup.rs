/// Result returned when select popup closes
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SelectResult {
    /// User selected an item
    Selected(String),
    /// User pressed Enter but no items matched the filter
    NoneSelected,
    /// User cancelled the popup (Esc)
    Cancelled,
}

/// Context for what action the select popup is performing
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SelectContext {
    /// Selecting a branch to checkout
    CheckoutBranch,
    /// Selecting an upstream to push to
    PushUpstream,
    /// Selecting an upstream to fetch from
    FetchUpstream,
    /// Selecting a branch to delete
    DeleteBranch,
    /// Selecting a remote to push all tags to
    PushAllTags,
}

/// State for the select popup (fuzzy finder style)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectPopupState {
    /// Display title (e.g., "Checkout", "Switch Branch")
    pub title: String,
    /// All available options (unfiltered)
    pub all_options: Vec<String>,
    /// Indices of options matching current filter
    pub filtered_indices: Vec<usize>,
    /// Current filter text
    pub input_text: String,
    /// Currently selected index in the filtered list (0-based)
    pub selected_index: usize,
}

impl SelectPopupState {
    /// Create a new select popup state with the given title and options
    pub fn new(title: String, options: Vec<String>) -> Self {
        let filtered_indices: Vec<usize> = (0..options.len()).collect();
        Self {
            title,
            all_options: options,
            filtered_indices,
            input_text: String::new(),
            selected_index: 0,
        }
    }

    /// Returns the currently selected item, if any
    pub fn selected_item(&self) -> Option<&str> {
        self.filtered_indices
            .get(self.selected_index)
            .and_then(|&idx| self.all_options.get(idx))
            .map(|s| s.as_str())
    }

    /// Updates filtered_indices based on current input_text (case-insensitive substring)
    pub fn update_filter(&mut self) {
        let query = self.input_text.to_lowercase();
        self.filtered_indices = self
            .all_options
            .iter()
            .enumerate()
            .filter(|(_, opt)| opt.to_lowercase().contains(&query))
            .map(|(idx, _)| idx)
            .collect();
        // Reset selection to first match
        self.selected_index = 0;
    }

    /// Number of filtered hits
    pub fn filtered_count(&self) -> usize {
        self.filtered_indices.len()
    }

    /// Move selection up
    pub fn move_up(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }

    /// Move selection down
    pub fn move_down(&mut self) {
        if self.selected_index + 1 < self.filtered_indices.len() {
            self.selected_index += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_select_popup_state_new() {
        let options = vec![
            "main".to_string(),
            "feature".to_string(),
            "develop".to_string(),
        ];
        let state = SelectPopupState::new("Checkout".to_string(), options.clone());

        assert_eq!(state.title, "Checkout");
        assert_eq!(state.all_options, options);
        assert_eq!(state.filtered_indices, vec![0, 1, 2]);
        assert_eq!(state.input_text, "");
        assert_eq!(state.selected_index, 0);
    }

    #[test]
    fn test_select_popup_state_selected_item() {
        let options = vec![
            "main".to_string(),
            "feature".to_string(),
            "develop".to_string(),
        ];
        let mut state = SelectPopupState::new("Checkout".to_string(), options);

        // Initially selects first item
        assert_eq!(state.selected_item(), Some("main"));

        // Move selection
        state.selected_index = 1;
        assert_eq!(state.selected_item(), Some("feature"));

        state.selected_index = 2;
        assert_eq!(state.selected_item(), Some("develop"));
    }

    #[test]
    fn test_select_popup_state_selected_item_empty() {
        let state = SelectPopupState::new("Checkout".to_string(), vec![]);
        assert_eq!(state.selected_item(), None);
    }

    #[test]
    fn test_select_popup_state_update_filter() {
        let options = vec![
            "main".to_string(),
            "feature/auth".to_string(),
            "feature/ui".to_string(),
            "develop".to_string(),
        ];
        let mut state = SelectPopupState::new("Checkout".to_string(), options);

        // Filter for "feature"
        state.input_text = "feature".to_string();
        state.update_filter();
        assert_eq!(state.filtered_indices, vec![1, 2]);
        assert_eq!(state.selected_index, 0);
        assert_eq!(state.selected_item(), Some("feature/auth"));
    }

    #[test]
    fn test_select_popup_state_update_filter_case_insensitive() {
        let options = vec![
            "Main".to_string(),
            "FEATURE".to_string(),
            "develop".to_string(),
        ];
        let mut state = SelectPopupState::new("Checkout".to_string(), options);

        // Filter for "main" (lowercase) should match "Main"
        state.input_text = "main".to_string();
        state.update_filter();
        assert_eq!(state.filtered_indices, vec![0]);
        assert_eq!(state.selected_item(), Some("Main"));

        // Filter for "DEVELOP" (uppercase) should match "develop"
        state.input_text = "DEVELOP".to_string();
        state.update_filter();
        assert_eq!(state.filtered_indices, vec![2]);
        assert_eq!(state.selected_item(), Some("develop"));
    }

    #[test]
    fn test_select_popup_state_update_filter_no_matches() {
        let options = vec!["main".to_string(), "feature".to_string()];
        let mut state = SelectPopupState::new("Checkout".to_string(), options);

        state.input_text = "nonexistent".to_string();
        state.update_filter();
        assert!(state.filtered_indices.is_empty());
        assert_eq!(state.selected_item(), None);
    }

    #[test]
    fn test_select_popup_state_move_up() {
        let options = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        let mut state = SelectPopupState::new("Test".to_string(), options);

        state.selected_index = 2;
        state.move_up();
        assert_eq!(state.selected_index, 1);

        state.move_up();
        assert_eq!(state.selected_index, 0);

        // Should not go below 0
        state.move_up();
        assert_eq!(state.selected_index, 0);
    }

    #[test]
    fn test_select_popup_state_move_down() {
        let options = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        let mut state = SelectPopupState::new("Test".to_string(), options);

        assert_eq!(state.selected_index, 0);
        state.move_down();
        assert_eq!(state.selected_index, 1);

        state.move_down();
        assert_eq!(state.selected_index, 2);

        // Should not go beyond the last item
        state.move_down();
        assert_eq!(state.selected_index, 2);
    }

    #[test]
    fn test_select_popup_state_filtered_count() {
        let options = vec![
            "main".to_string(),
            "feature/a".to_string(),
            "feature/b".to_string(),
        ];
        let mut state = SelectPopupState::new("Test".to_string(), options);

        assert_eq!(state.filtered_count(), 3);

        state.input_text = "feature".to_string();
        state.update_filter();
        assert_eq!(state.filtered_count(), 2);

        state.input_text = "xyz".to_string();
        state.update_filter();
        assert_eq!(state.filtered_count(), 0);
    }

    #[test]
    fn test_select_popup_state_filter_resets_selection() {
        let options = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        let mut state = SelectPopupState::new("Test".to_string(), options);

        state.selected_index = 2;
        state.input_text = "b".to_string();
        state.update_filter();

        // Selection should reset to 0 after filter
        assert_eq!(state.selected_index, 0);
    }
}
