use std::time::{Duration, Instant};

use crate::{
    git::revert::{self, CommitResult, any_is_merge_commit, parent_count},
    model::{
        Model, Toast, ToastStyle,
        popup::{PopupContent, PopupContentCommand},
        select_popup::{OnSelect, SelectPopupState},
    },
    msg::{Message, RevertCommand, update::pty_helper::execute_pty_command},
};

const TOAST_DURATION: Duration = Duration::from_secs(5);

pub fn update(model: &mut Model, cmd: RevertCommand) -> Option<Message> {
    match cmd {
        RevertCommand::Commits(hashes) => commits(model, hashes),
        RevertCommand::NoCommit(hashes) => no_commit(model, hashes),
        RevertCommand::CommitsWithMainline {
            hashes,
            mainline,
            no_commit,
        } => commits_with_mainline(model, hashes, mainline, no_commit),
        RevertCommand::Continue => continue_revert(model),
        RevertCommand::Skip => skip_revert(model),
        RevertCommand::Abort => abort_revert(model),
    }
}

fn commits(model: &mut Model, hashes: Vec<String>) -> Option<Message> {
    if hashes.is_empty() {
        return None;
    }
    if any_is_merge_commit(&model.workdir, &hashes) {
        show_mainline_popup(model, hashes, false);
        return None;
    }
    let mut args = vec!["revert".to_string(), "--no-edit".to_string()];
    args.extend(hashes);
    execute_pty_command(model, args, "Revert".to_string())
}

fn no_commit(model: &mut Model, hashes: Vec<String>) -> Option<Message> {
    if hashes.is_empty() {
        return None;
    }
    if any_is_merge_commit(&model.workdir, &hashes) {
        show_mainline_popup(model, hashes, true);
        return None;
    }
    let mut args = vec!["revert".to_string(), "--no-commit".to_string()];
    args.extend(hashes);
    execute_pty_command(model, args, "Revert".to_string())
}

fn commits_with_mainline(
    model: &mut Model,
    hashes: Vec<String>,
    mainline: u8,
    no_commit: bool,
) -> Option<Message> {
    if hashes.is_empty() {
        return None;
    }
    let mainline_str = mainline.to_string();
    let flag = if no_commit {
        "--no-commit"
    } else {
        "--no-edit"
    };
    let mut args = vec![
        "revert".to_string(),
        "-m".to_string(),
        mainline_str,
        flag.to_string(),
    ];
    args.extend(hashes);
    execute_pty_command(model, args, "Revert".to_string())
}

fn show_mainline_popup(model: &mut Model, hashes: Vec<String>, no_commit: bool) {
    let options = if hashes.len() == 1 {
        let count = parent_count(&model.workdir, &hashes[0]);
        if count > 2 {
            (1..=count)
                .map(|n| format!("{}  parent {}", n, n))
                .collect()
        } else {
            vec![
                "1  first parent (branch merged into)".to_string(),
                "2  second parent (merged branch)".to_string(),
            ]
        }
    } else {
        vec![
            "1  first parent (branch merged into)".to_string(),
            "2  second parent (merged branch)".to_string(),
        ]
    };

    let state = SelectPopupState::new(
        "Replay merges relative to parent".to_string(),
        options,
        OnSelect::RevertMergeMainline { hashes, no_commit },
    );
    model.popup = Some(PopupContent::Command(PopupContentCommand::Select(state)));
}

fn continue_revert(model: &mut Model) -> Option<Message> {
    model.popup = None;
    match revert::run_revert_continue_with_editor(&model.workdir) {
        Ok(CommitResult { success, message }) => {
            model.toast = Some(Toast {
                message,
                style: if success {
                    ToastStyle::Success
                } else {
                    ToastStyle::Warning
                },
                expires_at: Instant::now() + TOAST_DURATION,
            });
        }
        Err(e) => {
            model.popup = Some(PopupContent::Error {
                message: e.to_string(),
            });
        }
    }
    Some(Message::Refresh)
}

fn skip_revert(model: &mut Model) -> Option<Message> {
    execute_pty_command(
        model,
        vec!["revert".to_string(), "--skip".to_string()],
        "Revert".to_string(),
    )
}

fn abort_revert(model: &mut Model) -> Option<Message> {
    execute_pty_command(
        model,
        vec!["revert".to_string(), "--abort".to_string()],
        "Revert".to_string(),
    )
}
