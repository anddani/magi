use std::time::Instant;

use crate::{
    git::{
        checkout::{
            BranchEntry, get_all_branches, get_branches, get_last_checked_out_branch,
            get_local_branches, get_remote_branches_for_upstream,
        },
        log::get_log_entries,
        open_pr::has_any_remote,
        push::{get_current_branch, get_local_tags, get_remotes},
    },
    model::{
        BranchSuggestion, LineContent, Model, Toast, ToastStyle,
        popup::{
            CommitSelectPopupState, ConfirmAction, ConfirmPopupState, PopupContent,
            PopupContentCommand, SelectContext, SelectPopupState,
        },
        suggestions_from_line,
    },
    msg::{FixupType, LogType, Message, SelectPopup, StashCommand, update::commit::TOAST_DURATION},
};

pub fn update(model: &mut Model, popup: SelectPopup) -> Option<Message> {
    match popup {
        SelectPopup::FetchUpstream => {
            show_upstream_select(model, "Fetch from", SelectContext::FetchUpstream)
        }
        SelectPopup::PullUpstream => {
            show_upstream_select(model, "Pull from", SelectContext::PullUpstream)
        }
        SelectPopup::PushUpstream => {
            show_upstream_select(model, "Push to", SelectContext::PushUpstream)
        }

        SelectPopup::FetchElsewhere => {
            show_remote_select(model, "Fetch from", SelectContext::FetchElsewhere)
        }
        SelectPopup::FetchPushRemote => show_remote_select(
            model,
            "Fetch from push remote",
            SelectContext::FetchPushRemote,
        ),
        SelectPopup::PushPushRemote => {
            show_remote_select(model, "Push to push remote", SelectContext::PushPushRemote)
        }
        SelectPopup::PullPushRemote => show_remote_select(
            model,
            "Pull from push remote",
            SelectContext::PullPushRemote,
        ),
        SelectPopup::PushAllTags => {
            show_remote_select(model, "Push tags to", SelectContext::PushAllTags)
        }

        SelectPopup::PushTag => show_tag_select(model),
        SelectPopup::FetchAnotherBranch => show_fetch_another_branch(model),
        SelectPopup::FetchAnotherBranchBranch(r) => show_fetch_another_branch_branch(model, r),

        SelectPopup::CheckoutBranch => show_checkout_branch(model, false),
        SelectPopup::CheckoutLocalBranch => show_checkout_branch(model, true),
        SelectPopup::DeleteBranch => show_delete_branch(model),
        SelectPopup::RenameBranch => show_rename_branch(model),
        SelectPopup::CreateNewBranch { checkout } => show_new_branch_base(model, checkout),

        SelectPopup::StashApply => show_stash_select(model, StashOp::Apply),
        SelectPopup::StashPop => show_stash_select(model, StashOp::Pop),
        SelectPopup::StashDrop => show_stash_select(model, StashOp::Drop),

        SelectPopup::FixupCommit(fixup_type) => show_fixup_commit(model, fixup_type),

        SelectPopup::RebaseElsewhere => show_rebase_elsewhere(model),

        SelectPopup::OpenPr => show_open_pr(model, false),
        SelectPopup::OpenPrWithTarget => show_open_pr(model, true),
        SelectPopup::OpenPrTarget(b) => show_open_pr_target(model, b),
    }
}

// ── Shared helpers ────────────────────────────────────────────────────────────

/// Shows a select popup for choosing an upstream branch (fetch/pull/push).
fn show_upstream_select(model: &mut Model, title: &str, context: SelectContext) -> Option<Message> {
    let local_branch = get_current_branch(&model.workdir)
        .ok()
        .flatten()
        .unwrap_or_default();

    let remotes = get_remotes(&model.git_info.repository);
    let default_remote = remotes.into_iter().next().unwrap_or_default();

    let suggested = if !default_remote.is_empty() && !local_branch.is_empty() {
        Some(format!("{}/{}", default_remote, local_branch))
    } else {
        None
    };

    let branches =
        get_remote_branches_for_upstream(&model.git_info.repository, suggested.as_deref());

    if branches.is_empty() {
        model.popup = Some(PopupContent::Error {
            message: "No remote branches found".to_string(),
        });
        return None;
    }

    model.select_context = Some(context);
    let state = SelectPopupState::new(title.to_string(), branches);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Select(state)));
    None
}

/// Shows a select popup for choosing a remote (fetch-elsewhere, push-remote, etc.).
fn show_remote_select(model: &mut Model, title: &str, context: SelectContext) -> Option<Message> {
    let remotes = get_remotes(&model.git_info.repository);

    if remotes.is_empty() {
        model.popup = Some(PopupContent::Error {
            message: "No remotes configured".to_string(),
        });
        return None;
    }

    model.select_context = Some(context);
    let state = SelectPopupState::new(title.to_string(), remotes);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Select(state)));
    None
}

// ── Individual handlers ───────────────────────────────────────────────────────

fn show_tag_select(model: &mut Model) -> Option<Message> {
    let tags = get_local_tags(&model.git_info.repository);

    if tags.is_empty() {
        model.popup = Some(PopupContent::Error {
            message: "No tags to push".to_string(),
        });
        return None;
    }

    model.select_context = Some(SelectContext::PushTag);
    let state = SelectPopupState::new("Push tag".to_string(), tags);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Select(state)));
    None
}

fn show_fetch_another_branch(model: &mut Model) -> Option<Message> {
    let remotes = get_remotes(&model.git_info.repository);

    if remotes.is_empty() {
        model.popup = Some(PopupContent::Error {
            message: "No remotes configured".to_string(),
        });
        return None;
    }

    if remotes.len() == 1 {
        return Some(Message::ShowSelectPopup(
            SelectPopup::FetchAnotherBranchBranch(remotes.into_iter().next().unwrap()),
        ));
    }

    model.select_context = Some(SelectContext::FetchAnotherBranchRemote);
    let state = SelectPopupState::new("Fetch branch from".to_string(), remotes);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Select(state)));
    None
}

fn show_fetch_another_branch_branch(model: &mut Model, remote: String) -> Option<Message> {
    let prefix = format!("{}/", remote);

    let branches: Vec<String> = get_all_branches(&model.git_info.repository)
        .into_iter()
        .filter_map(|b| match b {
            BranchEntry::Remote(name) if name.starts_with(&prefix) => Some(name),
            _ => None,
        })
        .collect();

    if branches.is_empty() {
        model.popup = Some(PopupContent::Error {
            message: format!("No branches found for remote '{}'", remote),
        });
        return None;
    }

    model.select_context = Some(SelectContext::FetchAnotherBranch);
    let state = SelectPopupState::new(format!("Fetch branch from {}", remote), branches);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Select(state)));
    None
}

/// Shows a checkout-branch select popup.
/// If `local_only` is true, only local branches are listed (CheckoutLocalBranch).
/// If `local_only` is false, all branches (local + remote) are listed (CheckoutBranch).
fn show_checkout_branch(model: &mut Model, local_only: bool) -> Option<Message> {
    let current_branch = model.git_info.current_branch();

    let mut branches: Vec<String> = if local_only {
        get_local_branches(&model.git_info.repository)
    } else {
        get_branches(&model.git_info.repository)
    }
    .into_iter()
    .filter(|b| current_branch.as_deref() != Some(b.as_str()))
    .collect();

    if branches.is_empty() {
        model.popup = Some(PopupContent::Error {
            message: if local_only {
                "No local branches found".to_string()
            } else {
                "No branches found".to_string()
            },
        });
        return None;
    }

    model.select_context = Some(SelectContext::CheckoutBranch);

    let preferred = model
        .ui_model
        .lines
        .get(model.ui_model.cursor_position)
        .and_then(|line| {
            let suggestions = suggestions_from_line(line);
            suggestions.into_iter().find(|s| {
                if local_only {
                    matches!(s, BranchSuggestion::LocalBranch(name)
                        if current_branch.as_deref() != Some(name.as_str()))
                } else {
                    match s {
                        BranchSuggestion::LocalBranch(name) => {
                            current_branch.as_deref() != Some(name.as_str())
                        }
                        _ => true,
                    }
                }
            })
        })
        .or_else(|| {
            let last = get_last_checked_out_branch(&model.git_info.repository);
            if local_only {
                last.filter(|b| branches.contains(b))
                    .map(BranchSuggestion::LocalBranch)
            } else {
                last.map(BranchSuggestion::LocalBranch)
            }
        });

    if let Some(ref preferred) = preferred {
        let name = preferred.name();
        if let Some(idx) = branches.iter().position(|b| b == name) {
            let branch = branches.remove(idx);
            branches.insert(0, branch);
        } else if !local_only {
            // For all-branches mode, insert revisions at the top even if not in list
            branches.insert(0, name.to_string());
        }
    }

    let title = if local_only {
        "Checkout local"
    } else {
        "Checkout"
    };
    let state = SelectPopupState::new(title.to_string(), branches);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Select(state)));
    None
}

fn show_delete_branch(model: &mut Model) -> Option<Message> {
    let mut branches: Vec<String> = get_branches(&model.git_info.repository);

    if branches.is_empty() {
        model.popup = Some(PopupContent::Error {
            message: "No branches found".to_string(),
        });
        return None;
    }

    model.select_context = Some(SelectContext::DeleteBranch);

    let current_branch = model.git_info.current_branch();
    let preferred = model
        .ui_model
        .lines
        .get(model.ui_model.cursor_position)
        .and_then(|line| {
            suggestions_from_line(line).into_iter().find(|s| match s {
                BranchSuggestion::LocalBranch(name) => {
                    current_branch.as_deref() != Some(name.as_str())
                }
                BranchSuggestion::RemoteBranch(_) => true,
                BranchSuggestion::Revision(_) => false,
            })
        });

    if let Some(ref preferred) = preferred {
        let name = preferred.name();
        if let Some(idx) = branches.iter().position(|b| b == name) {
            let branch = branches.remove(idx);
            branches.insert(0, branch);
        }
    }

    let state = SelectPopupState::new("Delete branch".to_string(), branches);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Select(state)));
    None
}

fn show_rename_branch(model: &mut Model) -> Option<Message> {
    let current_branch = model.git_info.current_branch();
    let mut branches: Vec<String> = get_local_branches(&model.git_info.repository);

    if branches.is_empty() {
        model.popup = Some(PopupContent::Error {
            message: "No local branches found".to_string(),
        });
        return None;
    }

    model.select_context = Some(SelectContext::RenameBranch);

    let preferred = model
        .ui_model
        .lines
        .get(model.ui_model.cursor_position)
        .and_then(|line| {
            let suggestions = suggestions_from_line(line);
            suggestions.into_iter().find(|s| {
                matches!(s, BranchSuggestion::LocalBranch(name)
                    if current_branch.as_deref() != Some(name.as_str()))
            })
        })
        .or_else(|| {
            get_last_checked_out_branch(&model.git_info.repository)
                .filter(|b| branches.contains(b))
                .map(BranchSuggestion::LocalBranch)
        });

    if let Some(ref preferred) = preferred {
        let name = preferred.name();
        if let Some(idx) = branches.iter().position(|b| b == name) {
            let branch = branches.remove(idx);
            branches.insert(0, branch);
        }
    }

    let state = SelectPopupState::new("Rename branch".to_string(), branches);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Select(state)));
    None
}

fn show_new_branch_base(model: &mut Model, checkout: bool) -> Option<Message> {
    let branches = get_branches(&model.git_info.repository);
    let tags = get_local_tags(&model.git_info.repository);

    let mut options: Vec<String> = Vec::new();

    let preferred = model
        .ui_model
        .lines
        .get(model.ui_model.cursor_position)
        .and_then(|line| {
            let suggestions = suggestions_from_line(line);
            suggestions.into_iter().next()
        })
        .or_else(|| {
            model
                .git_info
                .current_branch()
                .map(|b| BranchSuggestion::LocalBranch(b.to_string()))
        });

    if let Some(ref preferred) = preferred {
        options.push(preferred.name().to_string());
    }

    for branch in branches {
        if preferred
            .as_ref()
            .map(|p| p.name() != branch)
            .unwrap_or(true)
        {
            options.push(branch);
        }
    }

    for tag in tags {
        if preferred
            .as_ref()
            .map(|p| match p {
                BranchSuggestion::Revision(rev) => rev != &tag,
                _ => p.name() != tag,
            })
            .unwrap_or(true)
        {
            options.push(tag);
        }
    }

    if options.is_empty() {
        model.popup = Some(PopupContent::Error {
            message: "No references found".to_string(),
        });
        return None;
    }

    model.select_context = Some(SelectContext::CreateNewBranchBase { checkout });
    let state = SelectPopupState::new("Create branch starting at".to_string(), options);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Select(state)));
    None
}

// ── Stash ─────────────────────────────────────────────────────────────────────

enum StashOp {
    Apply,
    Pop,
    Drop,
}

fn show_stash_select(model: &mut Model, op: StashOp) -> Option<Message> {
    let cursor_pos = model.ui_model.cursor_position;

    // For Drop: check if cursor is on the "Stashes" section header first
    if let StashOp::Drop = op
        && let Some(line) = model.ui_model.lines.get(cursor_pos)
        && let LineContent::SectionHeader { title, .. } = &line.content
        && title == "Stashes"
    {
        model.popup = Some(PopupContent::Confirm(ConfirmPopupState {
            message: "Drop all stashes?".to_string(),
            on_confirm: ConfirmAction::DropStash("all".to_string()),
        }));
        return None;
    }

    // If cursor is on a stash line, act immediately (op-specific behaviour)
    if let Some(line) = model.ui_model.lines.get(cursor_pos)
        && let LineContent::Stash(entry) = &line.content
    {
        let stash_ref = format!("stash@{{{}}}", entry.index);
        return match op {
            StashOp::Apply => Some(Message::Stash(StashCommand::Apply(stash_ref))),
            StashOp::Pop => {
                let message = format!(
                    "Pop {}?",
                    if entry.message.is_empty() {
                        stash_ref.clone()
                    } else {
                        format!("{}: {}", stash_ref, entry.message)
                    }
                );
                model.popup = Some(PopupContent::Confirm(ConfirmPopupState {
                    message,
                    on_confirm: ConfirmAction::PopStash(stash_ref),
                }));
                None
            }
            StashOp::Drop => {
                let message = format!(
                    "Drop {}?",
                    if entry.message.is_empty() {
                        stash_ref.clone()
                    } else {
                        format!("{}: {}", stash_ref, entry.message)
                    }
                );
                model.popup = Some(PopupContent::Confirm(ConfirmPopupState {
                    message,
                    on_confirm: ConfirmAction::DropStash(stash_ref),
                }));
                None
            }
        };
    }

    // Otherwise collect all stash entries and show a selection popup
    let stashes: Vec<String> = model
        .ui_model
        .lines
        .iter()
        .filter_map(|line| {
            if let LineContent::Stash(entry) = &line.content {
                Some(format!("stash@{{{}}}: {}", entry.index, entry.message))
            } else {
                None
            }
        })
        .collect();

    if stashes.is_empty() {
        model.popup = Some(PopupContent::Error {
            message: "No stashes found".to_string(),
        });
        return None;
    }

    let (context, title) = match op {
        StashOp::Apply => (SelectContext::ApplyStash, "Apply stash"),
        StashOp::Pop => (SelectContext::PopStash, "Pop stash"),
        StashOp::Drop => (SelectContext::DropStash, "Drop stash"),
    };

    model.select_context = Some(context);
    let state = SelectPopupState::new(title.to_string(), stashes);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Select(state)));
    None
}

// ── Fixup ─────────────────────────────────────────────────────────────────────

fn show_fixup_commit(model: &mut Model, fixup_type: FixupType) -> Option<Message> {
    if let Ok(false) = model.git_info.has_staged_changes() {
        model.toast = Some(Toast {
            message: "Nothing staged to fixup".to_string(),
            style: ToastStyle::Warning,
            expires_at: Instant::now() + TOAST_DURATION,
        });
        return Some(Message::DismissPopup);
    }

    match get_log_entries(&model.git_info.repository, LogType::Current) {
        Ok(mut commits) => {
            commits.retain(|entry| entry.is_commit());
            commits.truncate(50);

            if commits.is_empty() {
                model.popup = Some(PopupContent::Error {
                    message: "No commits found in current branch".to_string(),
                });
                None
            } else {
                let title = match fixup_type {
                    FixupType::Fixup => "Fixup commit".to_string(),
                    FixupType::Squash => "Squash commit".to_string(),
                    FixupType::Alter => "Alter commit".to_string(),
                    FixupType::Augment => "Augment commit".to_string(),
                };
                let state = CommitSelectPopupState::new(title, commits);
                model.popup = Some(PopupContent::Command(PopupContentCommand::CommitSelect(
                    state,
                )));
                model.select_context = Some(SelectContext::FixupCommit(fixup_type));
                None
            }
        }
        Err(err) => {
            model.popup = Some(PopupContent::Error {
                message: format!("Failed to get recent commits: {}", err),
            });
            None
        }
    }
}

// ── Open PR ───────────────────────────────────────────────────────────────────

fn show_open_pr(model: &mut Model, with_target: bool) -> Option<Message> {
    model.popup = None;

    let current_branch = match model.git_info.current_branch() {
        Some(branch) => branch,
        None => {
            model.toast = Some(Toast {
                message: "Not checked out to a branch (detached HEAD)".to_string(),
                style: ToastStyle::Warning,
                expires_at: Instant::now() + TOAST_DURATION,
            });
            return None;
        }
    };

    let mut branches: Vec<String> = get_local_branches(&model.git_info.repository)
        .into_iter()
        .filter(|b| has_any_remote(&model.workdir, b, &model.git_info.repository))
        .collect();

    if branches.is_empty() {
        model.popup = Some(PopupContent::Error {
            message: "No branches with upstream found".to_string(),
        });
        return None;
    }

    let preferred = model
        .ui_model
        .lines
        .get(model.ui_model.cursor_position)
        .and_then(|line| {
            let suggestions = suggestions_from_line(line);
            suggestions.into_iter().find(|s| {
                matches!(s, BranchSuggestion::LocalBranch(name)
                    if branches.contains(name))
            })
        });

    let preferred_name = preferred
        .map(|s| s.name().to_string())
        .unwrap_or(current_branch);

    if let Some(idx) = branches.iter().position(|b| b == &preferred_name) {
        let branch = branches.remove(idx);
        branches.insert(0, branch);
    }

    model.select_context = Some(if with_target {
        SelectContext::OpenPrBranchWithTarget
    } else {
        SelectContext::OpenPrBranch
    });

    let state = SelectPopupState::new("Open PR".to_string(), branches);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Select(state)));
    None
}

fn show_open_pr_target(model: &mut Model, branch: String) -> Option<Message> {
    let branches: Vec<String> = get_local_branches(&model.git_info.repository)
        .into_iter()
        .filter(|b| b != &branch)
        .collect();

    if branches.is_empty() {
        model.popup = Some(PopupContent::Error {
            message: "No other local branches found".to_string(),
        });
        return None;
    }

    model.open_pr_branch = Some(branch);
    model.select_context = Some(SelectContext::OpenPrTarget);

    let state = SelectPopupState::new("Open PR to".to_string(), branches);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Select(state)));
    None
}

// ── Rebase ────────────────────────────────────────────────────────────────────

fn show_rebase_elsewhere(model: &mut Model) -> Option<Message> {
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

    // Otherwise show a commit select popup with all commits
    match get_log_entries(&model.git_info.repository, LogType::AllReferences) {
        Ok(mut commits) => {
            commits.retain(|entry| entry.is_commit());
            commits.truncate(100);

            if commits.is_empty() {
                model.popup = Some(PopupContent::Error {
                    message: "No commits found".to_string(),
                });
                None
            } else {
                let state = CommitSelectPopupState::new("Rebase elsewhere".to_string(), commits);
                model.popup = Some(PopupContent::Command(PopupContentCommand::CommitSelect(
                    state,
                )));
                model.select_context = Some(SelectContext::RebaseElsewhere);
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
