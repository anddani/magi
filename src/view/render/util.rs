use std::collections::HashSet;

use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
};

use crate::{
    config::Theme,
    i18n,
    model::{Model, arguments::PopupArgument},
};

pub fn argument_lines<'a, A: PopupArgument>(
    theme: &Theme,
    arg_mode: bool,
    selected: Option<&HashSet<A>>,
) -> Vec<Line<'a>> {
    let empty = HashSet::new();
    let selected = selected.unwrap_or(&empty);
    A::all()
        .iter()
        .map(|arg| {
            argument_line(
                theme,
                arg.key(),
                arg.description(),
                arg.flag(),
                arg_mode,
                selected.contains(arg),
            )
        })
        .collect()
}

pub fn column_title<'a>(title: &'a str, theme: &Theme) -> Line<'a> {
    let column_title_style = Style::default()
        .fg(theme.section_header)
        .add_modifier(Modifier::BOLD);

    Line::from(Span::styled(title, column_title_style))
}

pub fn push_remote_description<'a>(
    model: &Model,
    theme: &Theme,
    push_remote: &Option<String>,
) -> Line<'a> {
    let key_style = Style::default()
        .fg(theme.local_branch)
        .add_modifier(Modifier::BOLD);
    let desc_style = Style::default();
    let faded_style = Style::default().fg(Color::DarkGray);

    // When in arg_mode, fade the command text
    let cmd_key_style = if model.arg_mode {
        faded_style
    } else {
        key_style
    };
    let cmd_desc_style = if model.arg_mode {
        faded_style
    } else {
        desc_style
    };
    let current_branch = model.git_info.current_branch().unwrap_or_default();
    match push_remote {
        Some(remote) => {
            let remote_style = if model.arg_mode {
                faded_style
            } else {
                Style::default().fg(theme.remote_branch)
            };
            Line::from(vec![
                Span::styled(" p", cmd_key_style),
                Span::styled(" ", cmd_desc_style),
                Span::styled(format!("{}/{}", remote, current_branch), remote_style),
            ])
        }
        None => command_description(
            theme,
            model.arg_mode,
            "p",
            i18n::t().arg_push_remote_setting_it,
        ),
    }
}

pub fn upstream_description<'a>(
    theme: &Theme,
    arg_mode: bool,
    upstream: &Option<String>,
) -> Line<'a> {
    let key_style = Style::default()
        .fg(theme.local_branch)
        .add_modifier(Modifier::BOLD);
    let desc_style = Style::default();
    let faded_style = Style::default().fg(Color::DarkGray);

    // When in arg_mode, fade the command text
    let cmd_key_style = if arg_mode { faded_style } else { key_style };
    let cmd_desc_style = if arg_mode { faded_style } else { desc_style };
    match upstream {
        Some(upstream) => {
            // Upstream is set - show in remote branch color (or faded if in arg_mode)
            let upstream_style = if arg_mode {
                faded_style
            } else {
                Style::default().fg(theme.remote_branch)
            };
            Line::from(vec![
                Span::styled(" u", cmd_key_style),
                Span::styled(" ", cmd_desc_style),
                Span::styled(upstream.clone(), upstream_style),
            ])
        }
        None => command_description(theme, arg_mode, "u", i18n::t().arg_upstream_setting_it),
    }
}

pub fn command_description<'a>(
    theme: &Theme,
    arg_mode: bool,
    key: &'a str,
    description: &'a str,
) -> Line<'a> {
    let key_style = Style::default()
        .fg(theme.local_branch)
        .add_modifier(Modifier::BOLD);
    let desc_style = Style::default();
    let faded_style = Style::default().fg(Color::DarkGray);

    // When in arg_mode, fade the command text
    let cmd_key_style = if arg_mode { faded_style } else { key_style };
    let cmd_desc_style = if arg_mode { faded_style } else { desc_style };
    Line::from(vec![
        Span::styled(format!(" {} ", key), cmd_key_style),
        Span::styled(description, cmd_desc_style),
    ])
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
        Span::styled(" -", dash_style),
        Span::styled(key.to_string(), key_style),
        Span::styled(format!(" {description} ("), desc_style),
        Span::styled(flag, flag_style),
        Span::styled(")", desc_style),
    ])
}
