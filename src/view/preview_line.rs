use ratatui::{
    style::Style,
    text::{Line as TextLine, Span},
};

use crate::config::Theme;
use crate::model::PreviewLineType;
use crate::view::util::expand_tabs;

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
    let content = expand_tabs(content, 0);
    vec![TextLine::from(Span::styled(content, style))]
}
