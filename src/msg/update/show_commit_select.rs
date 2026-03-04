use std::time::Instant;

use crate::{
    git::log::get_log_entries,
    model::{
        Line, LineContent, Model, Toast, ToastStyle, ViewMode,
        popup::{ConfirmAction, ConfirmPopupState, PopupContent, SelectContext},
    },
    msg::{CommitSelect, FixupType, LogType, Message, update::commit::TOAST_DURATION},
};

pub fn update(model: &mut Model, commit_select: CommitSelect) -> Option<Message> {
    match commit_select {
        CommitSelect::FixupCommit(fixup_type) => show_select_fixup_commit(model, fixup_type),
        CommitSelect::RebaseElsewhere => show_select_rebase_elsewhere_commit(model),
    }
}

pub fn show_select_fixup_commit(model: &mut Model, fixup_type: FixupType) -> Option<Message> {
    if let Ok(false) = model.git_info.has_staged_changes() {
        model.toast = Some(Toast {
            message: "Nothing staged to fixup".to_string(),
            style: ToastStyle::Warning,
            expires_at: Instant::now() + TOAST_DURATION,
        });
        return Some(Message::DismissPopup);
    }

    show_log_select(
        model,
        LogType::Current,
        SelectContext::FixupCommit(fixup_type),
    )
}

pub fn show_select_rebase_elsewhere_commit(model: &mut Model) -> Option<Message> {
    let cursor_pos = model.ui_model.cursor_position;

    // If cursor is on a commit line, suggest it and ask for confirmation
    if let Some(line) = model.ui_model.lines.get(cursor_pos) {
        let hash = match &line.content {
            LineContent::Commit(commit_info) => Some(commit_info.hash.clone()),
            LineContent::LogLine(entry) => entry.hash.clone(),
            _ => None,
        };

        if let Some(hash) = hash {
            model.popup = None;
            model.popup = Some(PopupContent::Confirm(ConfirmPopupState {
                message: format!("Rebase onto {}?", hash),
                on_confirm: ConfirmAction::RebaseElsewhere(hash),
            }));
            return None;
        }
    }

    show_log_select(
        model,
        LogType::AllReferences,
        SelectContext::RebaseElsewhere,
    )
}

fn show_log_select(
    model: &mut Model,
    log_type: LogType,
    context: SelectContext,
) -> Option<Message> {
    // Otherwise show the log view in picking mode
    match get_log_entries(&model.git_info.repository, log_type) {
        Ok(mut commits) => {
            commits.retain(|entry| entry.is_commit());

            if commits.is_empty() {
                model.popup = Some(PopupContent::Error {
                    message: "No commits found".to_string(),
                });
                None
            } else {
                let lines: Vec<Line> = commits
                    .into_iter()
                    .map(|entry| Line {
                        content: LineContent::LogLine(entry),
                        section: None,
                    })
                    .collect();
                model.ui_model.lines = lines;
                model.ui_model.cursor_position = 0;
                model.ui_model.scroll_offset = 0;
                model.view_mode = ViewMode::Log(log_type, true);
                model.popup = None;
                model.select_context = Some(context);
                None
            }
        }
        Err(err) => {
            model.popup = Some(PopupContent::Error {
                message: format!("Failed to get commits: {}", err),
            });
            None
        }
    }
}
