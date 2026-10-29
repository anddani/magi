use ratatui::{
    style::{Modifier, Style},
    text::{Line, Span},
};

use super::popup_content::{CommandPopupContent, PopupColumn, PopupColumnTitle, PopupRow};
use crate::{
    config::Theme,
    git::{config::get_local_config, push::get_remotes},
    i18n,
    model::{Model, popup::ConfigurePopupState},
};

pub fn content<'a>(
    theme: &Theme,
    model: &Model,
    state: &ConfigurePopupState,
) -> CommandPopupContent<'a> {
    let t = i18n::t();
    let branch = &state.branch;
    let workdir = &model.workdir;
    let remotes = get_remotes(&model.git_info.repository);
    let remote_choices: Vec<&str> = remotes.iter().map(|s| s.as_str()).collect();

    // ── Section 1: branch-scoped variables ──────────────────────────────────
    let desc_key = format!("branch.{}.description", branch);
    let merge_key = format!("branch.{}.merge", branch);
    let remote_key = format!("branch.{}.remote", branch);
    let rebase_key = format!("branch.{}.rebase", branch);
    let push_remote_key = format!("branch.{}.pushRemote", branch);

    let pull_rebase = get_local_config(workdir, "pull.rebase");
    let push_default = get_local_config(workdir, "remote.pushDefault");

    let pad1 = [
        &desc_key,
        &merge_key,
        &remote_key,
        &rebase_key,
        &push_remote_key,
    ]
    .iter()
    .map(|s| s.len())
    .max()
    .unwrap_or(0);

    let branch_col = PopupColumn {
        title: Some(PopupColumnTitle::Styled(scope_title(theme, branch))),
        content: vec![
            text_value_line(
                theme,
                "d",
                &desc_key,
                pad1,
                get_local_config(workdir, &desc_key),
            ),
            text_value_line(
                theme,
                "u",
                &merge_key,
                pad1,
                get_local_config(workdir, &merge_key),
            ),
            text_value_line(
                theme,
                "",
                &remote_key,
                pad1,
                get_local_config(workdir, &remote_key),
            ),
            choice_line(
                theme,
                "r",
                &rebase_key,
                pad1,
                &["true", "false"],
                get_local_config(workdir, &rebase_key),
                Some(format!(
                    "default:{}",
                    pull_rebase.clone().unwrap_or_else(|| "false".to_string())
                )),
            ),
            choice_line(
                theme,
                "p",
                &push_remote_key,
                pad1,
                &remote_choices,
                get_local_config(workdir, &push_remote_key),
                push_default.clone().map(|v| format!("default:{}", v)),
            ),
        ],
    };

    // ── Section 2: repository defaults ──────────────────────────────────────
    let pad2 = ["pull.rebase", "remote.pushDefault"]
        .iter()
        .map(|s| s.len())
        .max()
        .unwrap_or(0);

    let defaults_col = PopupColumn {
        title: Some(t.col_configure_defaults.into()),
        content: vec![
            choice_line(
                theme,
                "R",
                "pull.rebase",
                pad2,
                &["true", "false"],
                pull_rebase,
                Some("default:false".to_string()),
            ),
            choice_line(
                theme,
                "P",
                "remote.pushDefault",
                pad2,
                &remote_choices,
                push_default,
                None,
            ),
        ],
    };

    // ── Section 3: branch-creation defaults ─────────────────────────────────
    let pad3 = ["branch.autoSetupMerge", "branch.autoSetupRebase"]
        .iter()
        .map(|s| s.len())
        .max()
        .unwrap_or(0);

    let creation_col = PopupColumn {
        title: Some(t.col_configure_creation.into()),
        content: vec![
            choice_line(
                theme,
                "a m",
                "branch.autoSetupMerge",
                pad3,
                &["always", "true", "false"],
                get_local_config(workdir, "branch.autoSetupMerge"),
                Some("default:true".to_string()),
            ),
            choice_line(
                theme,
                "a r",
                "branch.autoSetupRebase",
                pad3,
                &["always", "local", "remote", "never"],
                get_local_config(workdir, "branch.autoSetupRebase"),
                Some("default:never".to_string()),
            ),
        ],
    };

    CommandPopupContent {
        title: t.title_configure_branch,
        rows: vec![
            PopupRow {
                columns: vec![branch_col],
            },
            PopupRow {
                columns: vec![defaults_col],
            },
            PopupRow {
                columns: vec![creation_col],
            },
        ],
    }
}

/// Builds the styled "Configure <branch>" column title.
fn scope_title(theme: &Theme, branch: &str) -> Line<'static> {
    let title_style = Style::default()
        .fg(theme.section_header)
        .add_modifier(Modifier::BOLD);
    let branch_style = Style::default()
        .fg(theme.local_branch)
        .add_modifier(Modifier::BOLD);
    Line::from(vec![
        Span::styled(format!("{} ", i18n::t().col_configure), title_style),
        Span::styled(branch.to_string(), branch_style),
    ])
}

/// Renders the leading key field (padded to width 3 to fit `a m`/`a r`).
fn key_span(theme: &Theme, key: &str) -> Span<'static> {
    let key_style = Style::default()
        .fg(theme.local_branch)
        .add_modifier(Modifier::BOLD);
    Span::styled(format!(" {:<3} ", key), key_style)
}

/// Renders a variable whose value is free text, e.g. the branch description or
/// upstream merge ref. The value is shown green when set, dim `unset` otherwise.
fn text_value_line(
    theme: &Theme,
    key: &str,
    varname: &str,
    pad: usize,
    value: Option<String>,
) -> Line<'static> {
    let green = Style::default().fg(theme.diff_addition);
    let dim = Style::default().fg(theme.dim_text);
    let value_span = match value {
        Some(v) => Span::styled(v, green),
        None => Span::styled(i18n::t().configure_unset.to_string(), dim),
    };
    Line::from(vec![
        key_span(theme, key),
        Span::styled(format!("{:<pad$} ", varname), Style::default()),
        value_span,
    ])
}

/// Renders a variable whose value cycles through a fixed set of choices, e.g.
/// `[true|false|default:false]`. The active choice is green; the brackets,
/// separators and fallback element stay dim.
fn choice_line(
    theme: &Theme,
    key: &str,
    varname: &str,
    pad: usize,
    choices: &[&str],
    current: Option<String>,
    fallback: Option<String>,
) -> Line<'static> {
    let green = Style::default().fg(theme.diff_addition);
    let dim = Style::default().fg(theme.dim_text);

    let mut spans = vec![
        key_span(theme, key),
        Span::styled(format!("{:<pad$} ", varname), Style::default()),
        Span::styled("[", dim),
    ];
    for (i, choice) in choices.iter().enumerate() {
        if i > 0 {
            spans.push(Span::styled("|", dim));
        }
        let style = if current.as_deref() == Some(*choice) {
            green
        } else {
            dim
        };
        spans.push(Span::styled(choice.to_string(), style));
    }
    if let Some(fallback) = fallback {
        if !choices.is_empty() {
            spans.push(Span::styled("|", dim));
        }
        spans.push(Span::styled(fallback, dim));
    }
    spans.push(Span::styled("]", dim));
    Line::from(spans)
}
