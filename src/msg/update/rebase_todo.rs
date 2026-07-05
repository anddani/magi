use std::time::{Duration, Instant};

use crate::{
    git::rebase::{self, RebaseAction, RebaseTodoEntry},
    i18n,
    model::{
        Line, LineContent, Model, Toast, ToastStyle, ViewMode, popup::PopupContent,
        rebase_todo::RebaseTodoState,
    },
    msg::{Message, RebaseTodoMessage},
};

const TOAST_DURATION: Duration = Duration::from_secs(5);

/// Opens the interactive rebase todo editor for `base..HEAD` (base inclusive).
pub fn show(model: &mut Model, base: String) -> Option<Message> {
    let base_has_parent = rebase::commit_has_parent(&model.workdir, &base);
    match rebase::get_interactive_rebase_commits(&model.workdir, &base, base_has_parent) {
        Ok(entries) if entries.is_empty() => {
            model.popup = Some(PopupContent::Error {
                message: "No commits to rebase".to_string(),
            });
            None
        }
        Ok(entries) => {
            let state = RebaseTodoState::new(base, base_has_parent, entries);
            model.ui_model.lines = todo_lines(&state.entries);
            model.ui_model.cursor_position = 0;
            model.ui_model.scroll_offset = 0;
            model.ui_model.visual_mode_anchor = None;
            model.rebase_todo = Some(state);
            model.view_mode = ViewMode::RebaseTodo;
            model.popup = None;
            None
        }
        Err(e) => {
            model.popup = Some(PopupContent::Error {
                message: e.to_string(),
            });
            None
        }
    }
}

pub fn update(model: &mut Model, msg: RebaseTodoMessage) -> Option<Message> {
    match msg {
        RebaseTodoMessage::SetAction(action) => set_action(model, action),
        RebaseTodoMessage::MoveEntryUp => move_entry(model, true),
        RebaseTodoMessage::MoveEntryDown => move_entry(model, false),
        RebaseTodoMessage::Undo => undo(model),
        RebaseTodoMessage::Abort => abort(model),
    }
}

fn set_action(model: &mut Model, action: RebaseAction) -> Option<Message> {
    let index = model.ui_model.cursor_position;
    let state = model.rebase_todo.as_mut()?;
    if state.set_action(index, action) {
        // Auto-advance to the next entry, like Magit's rebase editor
        if index + 1 < state.entries.len() {
            model.ui_model.cursor_position = index + 1;
        }
        model.ui_model.lines = todo_lines(&state.entries);
    } else if index == 0 && action.is_fold() {
        model.toast = Some(Toast {
            message: "Cannot squash/fixup without a previous commit".to_string(),
            style: ToastStyle::Warning,
            expires_at: Instant::now() + TOAST_DURATION,
        });
    }
    None
}

fn move_entry(model: &mut Model, up: bool) -> Option<Message> {
    let index = model.ui_model.cursor_position;
    let state = model.rebase_todo.as_mut()?;
    let moved = if up {
        state.move_entry_up(index)
    } else {
        state.move_entry_down(index)
    };
    if moved {
        model.ui_model.lines = todo_lines(&state.entries);
        // Keep the cursor on the entry that was moved
        model.ui_model.cursor_position = if up { index - 1 } else { index + 1 };
    }
    None
}

fn undo(model: &mut Model) -> Option<Message> {
    let state = model.rebase_todo.as_mut()?;
    if state.undo() {
        model.ui_model.lines = todo_lines(&state.entries);
        let max_pos = model.ui_model.lines.len().saturating_sub(1);
        if model.ui_model.cursor_position > max_pos {
            model.ui_model.cursor_position = max_pos;
        }
    }
    None
}

/// Closes the editor without touching the repository. Nothing has been
/// executed yet, so this simply returns to the status view.
fn abort(model: &mut Model) -> Option<Message> {
    model.rebase_todo = None;
    model.view_mode = ViewMode::Status;
    Some(Message::Refresh)
}

/// Builds the UI lines from the todo entries (one line per entry), followed
/// by a keybinding hint block (one line per key).
pub fn todo_lines(entries: &[RebaseTodoEntry]) -> Vec<Line> {
    let t = i18n::t();
    let mut lines: Vec<Line> = entries
        .iter()
        .map(|entry| Line {
            content: LineContent::RebaseTodoLine(entry.clone()),
            section: None,
        })
        .collect();

    lines.push(Line {
        content: LineContent::EmptyLine,
        section: None,
    });
    let hints = [
        ("p", t.rebase_hint_pick),
        ("r", t.rebase_hint_reword),
        ("e", t.rebase_hint_edit),
        ("s", t.rebase_hint_squash),
        ("f", t.rebase_hint_fixup),
        ("d", t.rebase_hint_drop),
        ("K/J", t.rebase_hint_move),
        ("u", t.rebase_hint_undo),
        ("RET", t.rebase_hint_show),
        ("R", t.rebase_hint_confirm),
        ("q", t.rebase_hint_abort),
    ];
    for (key, description) in hints {
        lines.push(Line {
            content: LineContent::RebaseTodoHint { key, description },
            section: None,
        });
    }
    lines
}
