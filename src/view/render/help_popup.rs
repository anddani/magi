use super::popup_content::CommandPopupContent;

use ratatui::{
    style::Style,
    text::{Line, Span},
};

use crate::{
    config::Theme,
    i18n,
    view::render::{
        popup_content::{PopupColumn, PopupRow},
        util::command_description,
    },
};

pub fn content(theme: &Theme) -> CommandPopupContent<'static> {
    let t = i18n::t();

    let command_popup_col_1 = PopupColumn {
        title: Some(t.col_commands.into()),
        content: vec![
            command_description(theme, false, "A", t.cmd_apply),
            command_description(theme, false, "b", t.cmd_branch),
            command_description(theme, false, "c", t.cmd_commit),
            command_description(theme, false, "f", t.cmd_fetch),
            command_description(theme, false, "l", t.cmd_log),
            command_description(theme, false, "m", t.cmd_merge),
            command_description(theme, false, "F", t.cmd_pull),
            command_description(theme, false, "p", t.cmd_push),
        ],
    };

    let command_popup_col_2 = PopupColumn {
        title: Some("".into()),
        content: vec![
            command_description(theme, false, "r", t.cmd_rebase),
            command_description(theme, false, "O", t.cmd_reset),
            command_description(theme, false, "_", t.cmd_revert),
            command_description(theme, false, "z", t.cmd_stash),
            command_description(theme, false, "t", t.cmd_tag),
            command_description(theme, false, "w", t.cmd_worktree),
        ],
    };

    let applying_changes_col = PopupColumn {
        title: Some(t.col_applying_changes.into()),
        content: vec![
            command_description(theme, false, "a", t.cmd_apply),
            command_description(theme, false, "s", t.cmd_stage),
            command_description(theme, false, "S", t.cmd_stage_all),
            command_description(theme, false, "u", t.cmd_unstage),
            command_description(theme, false, "U", t.cmd_unstage_all),
            command_description(theme, false, "x", t.cmd_discard),
            command_description(theme, false, "-", t.cmd_reverse),
        ],
    };

    let general_col = PopupColumn {
        title: Some(t.col_general.into()),
        content: vec![
            command_description(theme, false, "q", t.cmd_quit),
            command_description(theme, false, "Ctrl+r/gr", t.cmd_refresh),
            command_description(theme, false, "?/h", t.cmd_show_help),
            command_description(theme, false, "j/Down", t.cmd_move_down),
            command_description(theme, false, "k/Up", t.cmd_move_up),
            command_description(theme, false, "Ctrl+d", t.cmd_half_page_down),
            command_description(theme, false, "Ctrl+u", t.cmd_half_page_up),
            command_description(theme, false, "gg", t.cmd_go_first_line),
            command_description(theme, false, "G", t.cmd_go_last_line),
            command_description(theme, false, "Ctrl+e", t.cmd_scroll_down),
            command_description(theme, false, "Ctrl+y", t.cmd_scroll_up),
            command_description(theme, false, "Tab", t.cmd_toggle_section),
            command_description(theme, false, "V", t.cmd_visual_mode),
        ],
    };

    let version_row = PopupRow {
        columns: vec![PopupColumn {
            title: None,
            content: vec![Line::from(Span::styled(
                t.fmt1(t.help_version_fmt, env!("CARGO_PKG_VERSION")),
                Style::default().fg(theme.dim_text),
            ))],
        }],
    };

    CommandPopupContent {
        title: t.popup_help,
        rows: vec![
            PopupRow {
                columns: vec![
                    command_popup_col_1,
                    command_popup_col_2,
                    PopupColumn {
                        title: Some("    ".into()),
                        content: vec![],
                    },
                    applying_changes_col,
                    PopupColumn {
                        title: Some("    ".into()),
                        content: vec![],
                    },
                    general_col,
                ],
            },
            version_row,
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn line_text(line: &Line) -> String {
        line.spans.iter().map(|s| s.content.as_ref()).collect()
    }

    #[test]
    fn last_row_shows_crate_version() {
        let theme = Theme::default();
        let popup = content(&theme);

        let version_line = popup
            .rows
            .last()
            .and_then(|row| row.columns.first())
            .and_then(|col| col.content.first())
            .expect("help popup should end with a version line");

        assert_eq!(
            line_text(version_line),
            format!("Magi version {}", env!("CARGO_PKG_VERSION"))
        );
    }
}
