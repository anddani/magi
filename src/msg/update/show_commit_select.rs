use std::time::Instant;

use crate::{
    git::{log::get_log_entries, rebase::get_upstream_merge_base},
    i18n,
    model::{
        Line, LineContent, Model, Toast, ToastStyle, ViewMode,
        arguments::DEFAULT_LOG_LIMIT,
        popup::{ConfirmAction, ConfirmPopupState, PopupContent},
        select_popup::OnSelect,
    },
    msg::{
        CommitSelect, FixupType, LogType, Message, RebaseCommand, update::commit::TOAST_DURATION,
    },
};

pub fn update(model: &mut Model, commit_select: CommitSelect) -> Option<Message> {
    match commit_select {
        CommitSelect::FixupCommit(fixup_type) => show_select_fixup_commit(model, fixup_type),
        CommitSelect::RebaseElsewhere => show_select_rebase_elsewhere_commit(model),
        CommitSelect::RebaseInteractive => show_select_rebase_interactive_commit(model),
        CommitSelect::RebaseSubset { newbase } => show_log_select(
            model,
            LogType::Current,
            OnSelect::RebaseSubsetStart { newbase },
        ),
        CommitSelect::ModifyCommit => show_select_modify_commit(model),
        CommitSelect::RewordCommit => show_select_reword_commit(model),
        CommitSelect::RemoveCommit => show_select_remove_commit(model),
        CommitSelect::Autosquash => show_select_autosquash(model),
        CommitSelect::ReviseCommit => show_select_revise_commit(model),
    }
}

pub fn show_select_modify_commit(model: &mut Model) -> Option<Message> {
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
                message: format!("Modify commit {}?", hash),
                on_confirm: ConfirmAction::ModifyCommit(hash),
            }));
            return None;
        }
    }

    show_log_select(model, LogType::Current, OnSelect::ModifyCommit)
}

pub fn show_select_reword_commit(model: &mut Model) -> Option<Message> {
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
                message: format!("Reword commit {}?", hash),
                on_confirm: ConfirmAction::RewordCommit(hash),
            }));
            return None;
        }
    }

    show_log_select(model, LogType::Current, OnSelect::RewordCommit)
}

pub fn show_select_remove_commit(model: &mut Model) -> Option<Message> {
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
                message: format!("Remove commit {}?", hash),
                on_confirm: ConfirmAction::RemoveCommit(hash),
            }));
            return None;
        }
    }

    show_log_select(model, LogType::Current, OnSelect::RemoveCommit)
}

/// With an upstream configured, autosquash runs on the upstream merge base
/// right away (like magit). Otherwise the user picks the first commit that
/// fixups may be squashed into.
pub fn show_select_autosquash(model: &mut Model) -> Option<Message> {
    if let Some(base) = get_upstream_merge_base(&model.workdir) {
        model.popup = None;
        return Some(Message::Rebase(RebaseCommand::Autosquash(base)));
    }

    show_log_select(model, LogType::Current, OnSelect::AutosquashCommit)
}

pub fn show_select_fixup_commit(model: &mut Model, fixup_type: FixupType) -> Option<Message> {
    if let Ok(false) = model.git_info.has_staged_changes() {
        model.toast = Some(Toast {
            message: i18n::t().err_nothing_staged_to_fixup.to_string(),
            style: ToastStyle::Warning,
            expires_at: Instant::now() + TOAST_DURATION,
        });
        return Some(Message::DismissPopup);
    }

    show_log_select(model, LogType::Current, OnSelect::FixupCommit(fixup_type))
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

    show_log_select(model, LogType::AllReferences, OnSelect::RebaseElsewhere)
}

pub fn show_select_rebase_interactive_commit(model: &mut Model) -> Option<Message> {
    let cursor_pos = model.ui_model.cursor_position;

    // With the cursor on a commit line, open the todo editor for it directly
    if let Some(line) = model.ui_model.lines.get(cursor_pos) {
        let hash = match &line.content {
            LineContent::Commit(commit_info) => Some(commit_info.hash.clone()),
            LineContent::LogLine(entry) => entry.hash.clone(),
            _ => None,
        };

        if let Some(hash) = hash {
            model.popup = None;
            return Some(Message::ShowRebaseTodo(hash));
        }
    }

    show_log_select(model, LogType::Current, OnSelect::RebaseInteractive)
}

pub fn show_select_revise_commit(model: &mut Model) -> Option<Message> {
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
                message: format!("Revise commit {}?", hash),
                on_confirm: ConfirmAction::ReviseCommit(hash),
            }));
            return None;
        }
    }

    show_log_select(model, LogType::Current, OnSelect::ReviseCommit)
}

fn show_log_select(model: &mut Model, log_type: LogType, on_select: OnSelect) -> Option<Message> {
    // Commit picking always shows the graph and refnames, without color
    match get_log_entries(
        &model.git_info.repository,
        &log_type,
        true,
        false,
        true,
        false,
        false,
        false,
        false,
        DEFAULT_LOG_LIMIT,
        None,
        None,
        None,
        None,
    ) {
        Ok(mut commits) => {
            commits.retain(|entry| entry.is_commit());

            if commits.is_empty() {
                model.popup = Some(PopupContent::Error {
                    message: i18n::t().err_no_commits_found.to_string(),
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
                model.save_log_return_state();
                model.ui_model.lines = lines;
                model.ui_model.cursor_position = 0;
                model.ui_model.scroll_offset = 0;
                model.view_mode = ViewMode::Log {
                    log_type,
                    picking: true,
                    graph: true,
                    color: false,
                    decorate: true,
                    show_header: false,
                    patch: false,
                    stat: false,
                    show_signature: false,
                    limit: DEFAULT_LOG_LIMIT,
                    author: None,
                    grep: None,
                    search_changes: None,
                    search_occurrences: None,
                };
                model.popup = None;
                model.log_pick_on_select = Some(on_select);
                None
            }
        }
        Err(err) => {
            model.popup = Some(PopupContent::Error {
                message: format!("{}: {}", i18n::t().err_get_commits_failed, err),
            });
            None
        }
    }
}
