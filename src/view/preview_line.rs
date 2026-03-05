use ratatui::{
    style::Style,
    text::{Line as TextLine, Span},
};

use crate::config::Theme;
use crate::model::PreviewLineType;

pub fn get_lines(
    content: &str,
    line_type: &PreviewLineType,
    theme: &Theme,
) -> Vec<TextLine<'static>> {
    let style = match line_type {
        PreviewLineType::Header => Style::default().fg(theme.text),
        PreviewLineType::DiffFileHeader => Style::default().fg(theme.diff_context),
        PreviewLineType::HunkHeader => Style::default().fg(theme.diff_hunk),
        PreviewLineType::Addition => Style::default().fg(theme.diff_addition),
        PreviewLineType::Deletion => Style::default().fg(theme.diff_deletion),
        PreviewLineType::Context => Style::default().fg(theme.diff_context),
    };
    vec![TextLine::from(Span::styled(content.to_string(), style))]
}
