/// A single editing operation on an [`InputField`], typically produced by
/// mapping a key event (see `keys::edit::edit_op_for_key`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditOp {
    Insert(char),
    DeleteBackward,
    DeleteForward,
    DeleteWordBackward,
    DeleteWordForward,
    DeleteToStart,
    DeleteToEnd,
    MoveLeft,
    MoveRight,
    MoveWordLeft,
    MoveWordRight,
    MoveToStart,
    MoveToEnd,
}

/// A single-line text input with a cursor, supporting readline-style editing.
///
/// The cursor is a character index in `0..=char_count`; all mutations are
/// UTF-8 safe.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct InputField {
    text: String,
    cursor: usize,
}

impl InputField {
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a field with the given text and the cursor at the end.
    pub fn from_text(text: impl Into<String>) -> Self {
        let text = text.into();
        let cursor = text.chars().count();
        Self { text, cursor }
    }

    pub fn as_str(&self) -> &str {
        &self.text
    }

    /// The cursor position as a character index.
    pub fn cursor(&self) -> usize {
        self.cursor
    }

    pub fn is_empty(&self) -> bool {
        self.text.is_empty()
    }

    pub fn clear(&mut self) {
        self.text.clear();
        self.cursor = 0;
    }

    /// Applies an edit operation. Returns `true` if the text changed
    /// (cursor movement alone returns `false`).
    pub fn apply(&mut self, op: EditOp) -> bool {
        match op {
            EditOp::Insert(c) => {
                let byte = self.byte_at(self.cursor);
                self.text.insert(byte, c);
                self.cursor += 1;
                true
            }
            EditOp::DeleteBackward => self.delete_range(self.cursor.saturating_sub(1), self.cursor),
            EditOp::DeleteForward => self.delete_range(self.cursor, self.cursor + 1),
            EditOp::DeleteWordBackward => self.delete_range(self.prev_word_boundary(), self.cursor),
            EditOp::DeleteWordForward => self.delete_range(self.cursor, self.next_word_boundary()),
            EditOp::DeleteToStart => self.delete_range(0, self.cursor),
            EditOp::DeleteToEnd => self.delete_range(self.cursor, self.char_count()),
            EditOp::MoveLeft => {
                self.cursor = self.cursor.saturating_sub(1);
                false
            }
            EditOp::MoveRight => {
                self.cursor = (self.cursor + 1).min(self.char_count());
                false
            }
            EditOp::MoveWordLeft => {
                self.cursor = self.prev_word_boundary();
                false
            }
            EditOp::MoveWordRight => {
                self.cursor = self.next_word_boundary();
                false
            }
            EditOp::MoveToStart => {
                self.cursor = 0;
                false
            }
            EditOp::MoveToEnd => {
                self.cursor = self.char_count();
                false
            }
        }
    }

    fn char_count(&self) -> usize {
        self.text.chars().count()
    }

    /// Byte offset of the given character index (clamped to the end).
    fn byte_at(&self, char_idx: usize) -> usize {
        self.text
            .char_indices()
            .nth(char_idx)
            .map(|(byte, _)| byte)
            .unwrap_or(self.text.len())
    }

    /// Deletes characters in the char-index range `from..to`, clamped.
    /// Returns `true` if anything was deleted.
    fn delete_range(&mut self, from: usize, to: usize) -> bool {
        let to = to.min(self.char_count());
        if from >= to {
            return false;
        }
        let start = self.byte_at(from);
        let end = self.byte_at(to);
        self.text.drain(start..end);
        if self.cursor >= to {
            self.cursor -= to - from;
        } else if self.cursor > from {
            self.cursor = from;
        }
        true
    }

    /// Readline word boundary: from the cursor, skip separators leftwards,
    /// then skip word characters.
    fn prev_word_boundary(&self) -> usize {
        let chars: Vec<char> = self.text.chars().collect();
        let mut pos = self.cursor.min(chars.len());
        while pos > 0 && !chars[pos - 1].is_alphanumeric() {
            pos -= 1;
        }
        while pos > 0 && chars[pos - 1].is_alphanumeric() {
            pos -= 1;
        }
        pos
    }

    /// Readline word boundary: from the cursor, skip separators rightwards,
    /// then skip word characters.
    fn next_word_boundary(&self) -> usize {
        let chars: Vec<char> = self.text.chars().collect();
        let mut pos = self.cursor.min(chars.len());
        while pos < chars.len() && !chars[pos].is_alphanumeric() {
            pos += 1;
        }
        while pos < chars.len() && chars[pos].is_alphanumeric() {
            pos += 1;
        }
        pos
    }
}

#[cfg(test)]
mod tests {
    use super::{EditOp::*, InputField};

    fn field(text: &str, cursor: usize) -> InputField {
        let mut f = InputField::from_text(text);
        f.cursor = cursor;
        f
    }

    #[test]
    fn from_text_places_cursor_at_end() {
        let f = InputField::from_text("héllo");
        assert_eq!(f.cursor(), 5);
    }

    #[test]
    fn insert_mid_text() {
        let mut f = field("hllo", 1);
        assert!(f.apply(Insert('e')));
        assert_eq!(f.as_str(), "hello");
        assert_eq!(f.cursor(), 2);
    }

    #[test]
    fn delete_backward_and_forward() {
        let mut f = field("abc", 1);
        assert!(f.apply(DeleteBackward));
        assert_eq!(f.as_str(), "bc");
        assert_eq!(f.cursor(), 0);
        assert!(f.apply(DeleteForward));
        assert_eq!(f.as_str(), "c");
        assert_eq!(f.cursor(), 0);
    }

    #[test]
    fn deletes_are_noops_at_boundaries() {
        let mut f = field("abc", 0);
        assert!(!f.apply(DeleteBackward));
        assert_eq!(f.as_str(), "abc");

        let mut f = field("abc", 3);
        assert!(!f.apply(DeleteForward));
        assert_eq!(f.as_str(), "abc");

        let mut f = InputField::new();
        assert!(!f.apply(DeleteWordBackward));
        assert!(!f.apply(DeleteToEnd));
        assert_eq!(f.as_str(), "");
    }

    #[test]
    fn delete_word_backward_skips_separators_then_word() {
        let mut f = InputField::from_text("foo-bar_baz");
        // `-` and `_` are separators; each kill removes one alphanumeric run
        // plus the separators between it and the cursor
        assert!(f.apply(DeleteWordBackward));
        assert_eq!(f.as_str(), "foo-bar_");
        assert_eq!(f.cursor(), 8);
        assert!(f.apply(DeleteWordBackward));
        assert_eq!(f.as_str(), "foo-");
        assert!(f.apply(DeleteWordBackward));
        assert_eq!(f.as_str(), "");
    }

    #[test]
    fn delete_word_backward_over_trailing_spaces() {
        let mut f = InputField::from_text("foo bar  ");
        assert!(f.apply(DeleteWordBackward));
        assert_eq!(f.as_str(), "foo ");
    }

    #[test]
    fn delete_word_forward() {
        let mut f = field("foo bar baz", 3);
        assert!(f.apply(DeleteWordForward));
        assert_eq!(f.as_str(), "foo baz");
        assert_eq!(f.cursor(), 3);
    }

    #[test]
    fn delete_to_start_and_end() {
        let mut f = field("hello world", 5);
        assert!(f.apply(DeleteToStart));
        assert_eq!(f.as_str(), " world");
        assert_eq!(f.cursor(), 0);

        let mut f = field("hello world", 5);
        assert!(f.apply(DeleteToEnd));
        assert_eq!(f.as_str(), "hello");
        assert_eq!(f.cursor(), 5);
    }

    #[test]
    fn movement_clamps_and_reports_no_text_change() {
        let mut f = field("ab", 0);
        assert!(!f.apply(MoveLeft));
        assert_eq!(f.cursor(), 0);
        assert!(!f.apply(MoveRight));
        assert_eq!(f.cursor(), 1);
        assert!(!f.apply(MoveToEnd));
        assert_eq!(f.cursor(), 2);
        assert!(!f.apply(MoveRight));
        assert_eq!(f.cursor(), 2);
        assert!(!f.apply(MoveToStart));
        assert_eq!(f.cursor(), 0);
    }

    #[test]
    fn word_movement() {
        let mut f = InputField::from_text("foo bar-baz");
        assert!(!f.apply(MoveWordLeft));
        assert_eq!(f.cursor(), 8); // before `baz`
        assert!(!f.apply(MoveWordLeft));
        assert_eq!(f.cursor(), 4); // before `bar`
        assert!(!f.apply(MoveWordLeft));
        assert_eq!(f.cursor(), 0);
        assert!(!f.apply(MoveWordRight));
        assert_eq!(f.cursor(), 3); // after `foo`
        assert!(!f.apply(MoveWordRight));
        assert_eq!(f.cursor(), 7); // after `bar`
        assert!(!f.apply(MoveWordRight));
        assert_eq!(f.cursor(), 11);
    }

    #[test]
    fn unicode_editing_is_char_based() {
        let mut f = InputField::from_text("héllo wörld");
        assert_eq!(f.cursor(), 11);
        assert!(f.apply(DeleteWordBackward));
        assert_eq!(f.as_str(), "héllo ");
        f.apply(MoveLeft);
        f.apply(MoveLeft);
        assert!(f.apply(Insert('ö')));
        assert_eq!(f.as_str(), "héllöo ");

        let mut f = InputField::from_text("a🎉b");
        f.apply(MoveLeft);
        f.apply(MoveLeft);
        assert!(f.apply(DeleteBackward));
        assert_eq!(f.as_str(), "🎉b");
    }

    #[test]
    fn clear_resets_cursor() {
        let mut f = InputField::from_text("abc");
        f.clear();
        assert_eq!(f.as_str(), "");
        assert_eq!(f.cursor(), 0);
        assert!(f.is_empty());
    }
}
