use ratatui::text::Line;

/// A column in a command popup
pub struct PopupColumn<'a> {
    /// Optional column header (displayed in bold)
    pub title: Option<&'a str>,
    /// The content lines for this column
    pub content: Vec<Line<'a>>,
}

/// Generalized content structure for command popups
pub struct CommandPopupContent<'a> {
    /// The popup window title
    pub title: &'a str,
    /// Columns to display (1 column = single layout, 2+ = split layout)
    pub columns: Vec<PopupColumn<'a>>,
}

impl<'a> CommandPopupContent<'a> {
    /// Create a single-column popup
    pub fn single_column(title: &'a str, content: Vec<Line<'a>>) -> Self {
        Self {
            title,
            columns: vec![PopupColumn {
                title: None,
                content,
            }],
        }
    }

    /// Create a two-column popup
    pub fn two_columns(
        title: &'a str,
        left_title: &'a str,
        left_content: Vec<Line<'a>>,
        right_title: &'a str,
        right_content: Vec<Line<'a>>,
    ) -> Self {
        Self {
            title,
            columns: vec![
                PopupColumn {
                    title: Some(left_title),
                    content: left_content,
                },
                PopupColumn {
                    title: Some(right_title),
                    content: right_content,
                },
            ],
        }
    }

    /// Calculate the maximum content height across all columns
    pub fn max_content_height(&self) -> usize {
        self.columns
            .iter()
            .map(|col| {
                let title_height = if col.title.is_some() { 1 } else { 0 };
                col.content.len() + title_height
            })
            .max()
            .unwrap_or(0)
    }
}
