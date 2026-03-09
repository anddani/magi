use ratatui::text::Line;

/// A column in a command popup row
pub struct PopupColumn<'a> {
    /// Optional column header (displayed in bold)
    pub title: Option<PopupColumnTitle<'a>>,
    /// The content lines for this column
    pub content: Vec<Line<'a>>,
}

pub enum PopupColumnTitle<'a> {
    Raw(&'a str),
    Styled(Line<'a>),
}

impl<'a> From<&'a str> for PopupColumnTitle<'a> {
    fn from(value: &'a str) -> Self {
        PopupColumnTitle::Raw(value)
    }
}

impl PopupColumnTitle<'_> {
    pub fn len(&self) -> usize {
        match self {
            PopupColumnTitle::Raw(s) => s.len(),
            PopupColumnTitle::Styled(line) => line.width(),
        }
    }
}

impl<'a> PopupColumn<'a> {
    /// Height in terminal rows (title line + content lines)
    pub fn height(&self) -> usize {
        let title_height = if self.title.is_some() { 1 } else { 0 };
        self.content.len() + title_height
    }

    /// Width in terminal columns (max of title length and widest content line)
    pub fn width(&self) -> usize {
        let title_width = self.title.as_ref().map(|t| t.len()).unwrap_or(0);
        let content_width = self
            .content
            .iter()
            .map(|line| line.width())
            .max()
            .unwrap_or(0);
        title_width.max(content_width)
    }
}

/// A row in a command popup, containing one or more side-by-side columns
pub struct PopupRow<'a> {
    /// Columns rendered side by side, each wrapping to its content width
    pub columns: Vec<PopupColumn<'a>>,
}

impl<'a> PopupRow<'a> {
    /// Height of this row (tallest column)
    pub fn height(&self) -> usize {
        self.columns
            .iter()
            .map(|col| col.height())
            .max()
            .unwrap_or(0)
    }
}

/// Generalized content structure for command popups
pub struct CommandPopupContent<'a> {
    /// The popup window title
    pub title: &'a str,
    /// Rows stacked vertically; each row holds one or more columns side by side
    pub rows: Vec<PopupRow<'a>>,
}

impl<'a> CommandPopupContent<'a> {
    /// Create a single-column popup
    pub fn single_column(title: &'a str, content: Vec<Line<'a>>) -> Self {
        Self {
            title,
            rows: vec![PopupRow {
                columns: vec![PopupColumn {
                    title: None,
                    content,
                }],
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
            rows: vec![PopupRow {
                columns: vec![
                    PopupColumn {
                        title: Some(left_title.into()),
                        content: left_content,
                    },
                    PopupColumn {
                        title: Some(right_title.into()),
                        content: right_content,
                    },
                ],
            }],
        }
    }

    /// Total content height (sum of every row's height plus 1-unit gaps between rows)
    pub fn total_content_height(&self) -> usize {
        let rows_height: usize = self.rows.iter().map(|row| row.height()).sum();
        let gaps = self.rows.len().saturating_sub(1);
        rows_height + gaps
    }
}
