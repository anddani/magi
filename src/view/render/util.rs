use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
};

use crate::config::Theme;

pub fn column_title<'a>(title: &'a str, theme: &Theme) -> Line<'a> {
    let column_title_style = Style::default()
        .fg(theme.section_header)
        .add_modifier(Modifier::BOLD);

    Line::from(Span::styled(title, column_title_style))
}

pub fn argument_line<'a>(
    theme: &Theme,
    key: char,
    description: &'a str,
    flag: &'a str,
    arg_mode: bool,
    selected: bool,
) -> Line<'a> {
    let faded_style = Style::default().fg(Color::DarkGray);
    let desc_style = Style::default();
    let key_style = Style::default()
        .fg(theme.diff_addition)
        .add_modifier(Modifier::BOLD);

    let dash_style = if arg_mode { faded_style } else { key_style };

    let flag_style = if selected {
        Style::default().fg(theme.diff_addition) // Green when selected
    } else {
        faded_style // Gray when not selected
    };
    Line::from(vec![
        Span::styled("-", dash_style),
        Span::styled(key.to_string(), key_style),
        Span::styled(format!(" {description} ("), desc_style),
        Span::styled(flag, flag_style),
        Span::styled(")", desc_style),
    ])
}
