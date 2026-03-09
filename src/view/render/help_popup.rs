use super::popup_content::CommandPopupContent;

use crate::{
    config::Theme,
    view::render::{
        popup_content::{PopupColumn, PopupRow},
        util::command_description,
    },
};

pub fn content(theme: &Theme) -> CommandPopupContent<'static> {
    let command_popup_col = PopupColumn {
        title: Some("Commands".into()),
        content: vec![
            command_description(theme, false, "b", "branch"),
            command_description(theme, false, "c", "commit"),
            command_description(theme, false, "f", "fetch"),
            command_description(theme, false, "F", "pull"),
            command_description(theme, false, "l", "log"),
            command_description(theme, false, "m", "merge"),
            command_description(theme, false, "p", "push"),
            command_description(theme, false, "t", "tag"),
            command_description(theme, false, "z", "stash"),
        ],
    };

    let applying_changes_col = PopupColumn {
        title: Some("Applying changes".into()),
        content: vec![
            command_description(theme, false, "s", "stage"),
            command_description(theme, false, "S", "stage all"),
            command_description(theme, false, "u", "unstage"),
            command_description(theme, false, "U", "unstage all"),
            command_description(theme, false, "x", "discard"),
        ],
    };

    let general_col = PopupColumn {
        title: Some("General".into()),
        content: vec![
            command_description(theme, false, "q", "        quit"),
            command_description(theme, false, "Ctrl+r/gr", "refresh"),
            command_description(theme, false, "?/h", "      show this help"),
            command_description(theme, false, "j/Down", "   move down"),
            command_description(theme, false, "k/Up", "     move up"),
            command_description(theme, false, "Ctrl+d", "   half page down"),
            command_description(theme, false, "Ctrl+u", "   half page up"),
            command_description(theme, false, "gg", "       go to first line"),
            command_description(theme, false, "G", "        go to last line"),
            command_description(theme, false, "Ctrl+e", "   scroll one line down"),
            command_description(theme, false, "Ctrl+y", "   scroll one line up"),
            command_description(
                theme,
                false,
                "Tab",
                "      toggle section collapsed/expanded",
            ),
            command_description(theme, false, "V", "        enter visual selection mode"),
        ],
    };

    CommandPopupContent {
        title: "Help",
        rows: vec![PopupRow {
            columns: vec![command_popup_col, applying_changes_col, general_col],
        }],
    }
}
