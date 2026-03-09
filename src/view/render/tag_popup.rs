use ratatui::{
    style::{Modifier, Style},
    text::{Line, Span},
};

use super::popup_content::{CommandPopupContent, PopupColumn, PopupRow};
use crate::{config::Theme, model::popup::TagPopupState};

pub fn content<'a>(theme: &Theme, _state: &'a TagPopupState) -> CommandPopupContent<'a> {
    let key_style = Style::default()
        .fg(theme.local_branch)
        .add_modifier(Modifier::BOLD);
    let desc_style = Style::default();

    let create = PopupColumn {
        title: Some("Create".into()),
        content: vec![Line::from(vec![
            Span::styled(" t", key_style),
            Span::styled(" tag", desc_style),
        ])],
    };

    let do_col = PopupColumn {
        title: Some("Do".into()),
        content: vec![Line::from(vec![
            Span::styled(" x", key_style),
            Span::styled(" delete", desc_style),
        ])],
    };

    CommandPopupContent {
        title: "Tag",
        rows: vec![PopupRow {
            columns: vec![create, do_col],
        }],
    }
}
