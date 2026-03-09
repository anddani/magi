use std::time::Instant;

use crate::{
    git::{
        CommitRefType,
        checkout::{
            BranchEntry, get_all_branches, get_branches, get_last_checked_out_branch,
            get_local_branches, get_remote_branches_for_upstream,
        },
        file_checkout::get_tracked_files,
        open_pr::has_any_remote,
        push::{get_current_branch, get_local_tags, get_remotes},
        worktree::get_checked_out_branches,
    },
    model::{
        BranchSuggestion, LineContent, Model, Toast, ToastStyle,
        popup::{
            ConfirmAction, ConfirmPopupState, PopupContent, PopupContentCommand, SelectContext,
            SelectPopupState,
        },
        suggestions_from_line,
    },
    msg::{Message, ResetMode, SelectPopup, StashCommand, update::commit::TOAST_DURATION},
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
        SelectPopup::PullElsewhere => {
            show_upstream_select(model, "Pull from", SelectContext::PullElsewhere)
        }
        SelectPopup::PushElsewhere => {
            show_upstream_select(model, "Push to", SelectContext::PushElsewhere)
        }
        SelectPopup::PushOtherBranchPick => show_push_other_branch_pick(model),
        SelectPopup::PushOtherBranchTarget(local) => show_upstream_select(
            model,
            "Push to",
            SelectContext::PushOtherBranchTarget(local),
        ),
        SelectPopup::PushRefspecRemotePick => show_remote_select(
            model,
            "Push to remote",
            SelectContext::PushRefspecRemotePick,
        ),
        SelectPopup::FetchRefspecRemotePick => show_remote_select(
            model,
            "Fetch from remote",
            SelectContext::FetchRefspecRemotePick,
        ),
        SelectPopup::PushMatching => show_remote_select(
            model,
            "Push matching branches to",
            SelectContext::PushMatching,
        ),
        SelectPopup::PushAllTags => {
            show_remote_select(model, "Push tags to", SelectContext::PushAllTags)
        }

        SelectPopup::PushTag => show_tag_select(model),
        SelectPopup::FetchAnotherBranch => show_fetch_another_branch(model),
        SelectPopup::FetchAnotherBranchBranch(r) => show_fetch_another_branch_branch(model, r),

        SelectPopup::CheckoutBranch => show_checkout_branch(model, false),
        SelectPopup::CheckoutLocalBranch => show_checkout_branch(model, true),
        SelectPopup::WorktreeCheckout => show_worktree_add(model, true),
        SelectPopup::WorktreeCreate => show_worktree_add(model, false),
        SelectPopup::DeleteBranch => show_delete_branch(model),
        SelectPopup::RenameBranch => show_rename_branch(model),
        SelectPopup::CreateNewBranch { checkout } => show_new_branch_base(model, checkout),

        SelectPopup::StashApply => show_stash_select(model, StashOp::Apply),
        SelectPopup::StashPop => show_stash_select(model, StashOp::Pop),
        SelectPopup::StashDrop => show_stash_select(model, StashOp::Drop),

        SelectPopup::OpenPr => show_open_pr(model, false),
        SelectPopup::OpenPrWithTarget => show_open_pr(model, true),
        SelectPopup::OpenPrTarget(b) => show_open_pr_target(model, b),

        SelectPopup::ResetBranchPick => show_reset_branch_pick(model),
        SelectPopup::ResetBranchTarget(branch) => show_reset_branch_target(model, branch),
        SelectPopup::Reset(reset_mode) => show_reset_ref_picker(model, reset_mode),
        SelectPopup::ResetIndex => show_reset_index_picker(model),
        SelectPopup::ResetWorktree => show_reset_worktree_picker(model),

        SelectPopup::FileCheckoutRevision => show_file_checkout_revision(model),
        SelectPopup::FileCheckoutFile(revision) => show_file_checkout_file(model, revision),

        SelectPopup::MergeElsewhere => show_merge_elsewhere(model),

        SelectPopup::CreateTagTarget(tag_name) => show_tag_target_select(model, tag_name),
        SelectPopup::DeleteTag => show_delete_tag(model),

        SelectPopup::PruneTagsRemotePick => show_prune_tags_remote_pick(model),
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

fn show_worktree_add(model: &mut Model, checkout: bool) -> Option<Message> {
    let checked_out = get_checked_out_branches(&model.workdir);
    let branches: Vec<String> = get_branches(&model.git_info.repository)
        .into_iter()
        .filter(|b| !checked_out.contains(b.as_str()))
        .collect();
    let tags = get_local_tags(&model.git_info.repository);

    let preferred = model
        .ui_model
        .lines
        .get(model.ui_model.cursor_position)
        .and_then(|line| {
            suggestions_from_line(line)
                .into_iter()
                .find(|s| !checked_out.contains(s.name()))
        });

    let mut options: Vec<String> = Vec::new();

    if let Some(ref preferred) = preferred {
        options.push(preferred.name().to_string());
    }

    for branch in &branches {
        if preferred
            .as_ref()
            .map(|p| p.name() != branch.as_str())
            .unwrap_or(true)
        {
            options.push(branch.clone());
        }
    }

    for tag in &tags {
        if preferred
            .as_ref()
            .map(|p| p.name() != tag.as_str())
            .unwrap_or(true)
        {
            options.push(tag.clone());
        }
    }

    if options.is_empty() {
        model.popup = Some(PopupContent::Error {
            message: "No branches or tags found".to_string(),
        });
        return None;
    }

    model.select_context = Some(SelectContext::WorktreeAdd { checkout });
    let title = if checkout {
        "Worktree checkout"
    } else {
        "Worktree create"
    };
    let state = SelectPopupState::new(title.to_string(), options);
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

// ── File checkout ──────────────────────────────────────────────────────────────

/// Step 1: pick the revision to checkout a file from (local branches, remote branches, tags).
/// The first suggestion from the cursor line is pre-selected if available.
fn show_file_checkout_revision(model: &mut Model) -> Option<Message> {
    let local_branches: Vec<String> = get_local_branches(&model.git_info.repository);

    let remote_branches: Vec<String> = get_all_branches(&model.git_info.repository)
        .into_iter()
        .filter_map(|b| match b {
            BranchEntry::Remote(name) => Some(name),
            _ => None,
        })
        .collect();

    let tags = get_local_tags(&model.git_info.repository);

    let preferred = model
        .ui_model
        .lines
        .get(model.ui_model.cursor_position)
        .and_then(|line| suggestions_from_line(line).into_iter().next());

    let mut options: Vec<String> = Vec::new();

    if let Some(ref preferred) = preferred {
        options.push(preferred.name().to_string());
    }

    for b in &local_branches {
        if preferred.as_ref().map(|p| p.name() != b).unwrap_or(true) {
            options.push(b.clone());
        }
    }
    for b in &remote_branches {
        if preferred
            .as_ref()
            .map(|p| p.name() != b.as_str())
            .unwrap_or(true)
        {
            options.push(b.clone());
        }
    }
    for tag in &tags {
        if preferred
            .as_ref()
            .map(|p| p.name() != tag.as_str())
            .unwrap_or(true)
        {
            options.push(tag.clone());
        }
    }

    if options.is_empty() {
        model.popup = Some(PopupContent::Error {
            message: "No references found".to_string(),
        });
        return None;
    }

    model.select_context = Some(SelectContext::FileCheckoutRevision);
    let state = SelectPopupState::new("Checkout file from revision".to_string(), options);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Select(state)));
    None
}

/// Step 2: pick the file to checkout from the given revision.
/// Cursor file (UnstagedFile, StagedFile, UntrackedFile) is pre-selected if available.
fn show_file_checkout_file(model: &mut Model, revision: String) -> Option<Message> {
    let mut files = get_tracked_files(&model.git_info.repository);

    if files.is_empty() {
        model.popup = Some(PopupContent::Error {
            message: "No tracked files found".to_string(),
        });
        return None;
    }

    // Pre-select file under cursor if it's a file line
    let cursor_file = model
        .ui_model
        .lines
        .get(model.ui_model.cursor_position)
        .and_then(|line| match &line.content {
            LineContent::UnstagedFile(fc) => Some(fc.path.clone()),
            LineContent::StagedFile(fc) => Some(fc.path.clone()),
            LineContent::UntrackedFile(path) => Some(path.clone()),
            _ => None,
        });

    if let Some(ref path) = cursor_file
        && let Some(idx) = files.iter().position(|f| f == path)
    {
        let file = files.remove(idx);
        files.insert(0, file);
    }

    model.select_context = Some(SelectContext::FileCheckoutFile(revision));
    let state = SelectPopupState::new("File to checkout".to_string(), files);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Select(state)));
    None
}

// ── Reset ─────────────────────────────────────────────────────────────────────

/// Step 1: pick which local branch to reset.
/// If the cursor is on a line with a local branch, that branch is pre-selected.
/// Falls back to the currently checked-out branch.
fn show_reset_branch_pick(model: &mut Model) -> Option<Message> {
    let mut branches = get_local_branches(&model.git_info.repository);

    if branches.is_empty() {
        model.popup = Some(PopupContent::Error {
            message: "No local branches found".to_string(),
        });
        return None;
    }

    // Prefer a local branch from the cursor line, fall back to current branch
    let preferred = model
        .ui_model
        .lines
        .get(model.ui_model.cursor_position)
        .and_then(|line| {
            suggestions_from_line(line).into_iter().find(
                |s| matches!(s, BranchSuggestion::LocalBranch(name) if branches.contains(name)),
            )
        })
        .or_else(|| {
            model
                .git_info
                .current_branch()
                .map(BranchSuggestion::LocalBranch)
        });

    if let Some(ref preferred) = preferred
        && let Some(idx) = branches.iter().position(|b| b == preferred.name())
    {
        let branch = branches.remove(idx);
        branches.insert(0, branch);
    }

    model.select_context = Some(SelectContext::ResetBranchPick);
    let state = SelectPopupState::new("Reset: select branch".to_string(), branches);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Select(state)));
    None
}

/// Step 2: pick what to reset the given branch to (local branches, remote branches, tags).
/// The first suggestion from the cursor line is prioritized — including a bare commit hash when
/// the line has no named refs (e.g. the cursor is on a plain commit line).
fn show_reset_branch_target(model: &mut Model, branch: String) -> Option<Message> {
    let local_branches: Vec<String> = get_local_branches(&model.git_info.repository)
        .into_iter()
        .filter(|b| b != &branch)
        .collect();

    let remote_branches: Vec<String> = get_all_branches(&model.git_info.repository)
        .into_iter()
        .filter_map(|b| match b {
            BranchEntry::Remote(name) => Some(name),
            _ => None,
        })
        .collect();

    let tags = get_local_tags(&model.git_info.repository);

    // Take the highest-priority suggestion from the cursor line.
    // suggestions_from_line returns: local branches first, then remote, then the hash.
    // So if the line has branch refs we get those; if it's a bare commit we get the hash.
    let preferred = model
        .ui_model
        .lines
        .get(model.ui_model.cursor_position)
        .and_then(|line| suggestions_from_line(line).into_iter().next());

    let mut options: Vec<String> = Vec::new();

    if let Some(ref preferred) = preferred {
        options.push(preferred.name().to_string());
    }

    for b in &local_branches {
        if preferred.as_ref().map(|p| p.name() != b).unwrap_or(true) {
            options.push(b.clone());
        }
    }
    for b in &remote_branches {
        if preferred
            .as_ref()
            .map(|p| p.name() != b.as_str())
            .unwrap_or(true)
        {
            options.push(b.clone());
        }
    }
    for tag in &tags {
        if preferred
            .as_ref()
            .map(|p| p.name() != tag.as_str())
            .unwrap_or(true)
        {
            options.push(tag.clone());
        }
    }

    if options.is_empty() {
        model.popup = Some(PopupContent::Error {
            message: "No references found".to_string(),
        });
        return None;
    }

    model.select_context = Some(SelectContext::ResetBranchTarget(branch));
    let state = SelectPopupState::new("Reset branch to".to_string(), options);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Select(state)));
    None
}

/// Pick a target ref/commit for a mixed reset of HEAD.
/// Includes local branches, remote branches, and tags.
/// Prioritises the first suggestion from the cursor line.
fn show_reset_ref_picker(model: &mut Model, reset_mode: ResetMode) -> Option<Message> {
    let local_branches = get_local_branches(&model.git_info.repository);

    let remote_branches: Vec<String> = get_all_branches(&model.git_info.repository)
        .into_iter()
        .filter_map(|b| match b {
            BranchEntry::Remote(name) => Some(name),
            _ => None,
        })
        .collect();

    let tags = get_local_tags(&model.git_info.repository);

    let preferred = model
        .ui_model
        .lines
        .get(model.ui_model.cursor_position)
        .and_then(|line| suggestions_from_line(line).into_iter().next());

    let mut options: Vec<String> = Vec::new();

    if let Some(ref preferred) = preferred {
        options.push(preferred.name().to_string());
    }

    for b in &local_branches {
        if preferred.as_ref().map(|p| p.name() != b).unwrap_or(true) {
            options.push(b.clone());
        }
    }
    for b in &remote_branches {
        if preferred
            .as_ref()
            .map(|p| p.name() != b.as_str())
            .unwrap_or(true)
        {
            options.push(b.clone());
        }
    }
    for tag in &tags {
        if preferred
            .as_ref()
            .map(|p| p.name() != tag.as_str())
            .unwrap_or(true)
        {
            options.push(tag.clone());
        }
    }

    if options.is_empty() {
        model.popup = Some(PopupContent::Error {
            message: "No references found".to_string(),
        });
        return None;
    }

    model.select_context = Some(SelectContext::Reset(reset_mode));
    let state = SelectPopupState::new(format!("{} reset to", reset_mode.name()), options);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Select(state)));
    None
}

/// Pick a target tree-ish for an index-only reset (`git reset <target> -- .`).
/// Includes local branches, remote branches, and tags.
/// Prioritises the first suggestion from the cursor line.
fn show_reset_index_picker(model: &mut Model) -> Option<Message> {
    let local_branches = get_local_branches(&model.git_info.repository);

    let remote_branches: Vec<String> = get_all_branches(&model.git_info.repository)
        .into_iter()
        .filter_map(|b| match b {
            BranchEntry::Remote(name) => Some(name),
            _ => None,
        })
        .collect();

    let tags = get_local_tags(&model.git_info.repository);

    let preferred = model
        .ui_model
        .lines
        .get(model.ui_model.cursor_position)
        .and_then(|line| suggestions_from_line(line).into_iter().next());

    let mut options: Vec<String> = Vec::new();

    if let Some(ref preferred) = preferred {
        options.push(preferred.name().to_string());
    }

    for b in &local_branches {
        if preferred.as_ref().map(|p| p.name() != b).unwrap_or(true) {
            options.push(b.clone());
        }
    }
    for b in &remote_branches {
        if preferred
            .as_ref()
            .map(|p| p.name() != b.as_str())
            .unwrap_or(true)
        {
            options.push(b.clone());
        }
    }
    for tag in &tags {
        if preferred
            .as_ref()
            .map(|p| p.name() != tag.as_str())
            .unwrap_or(true)
        {
            options.push(tag.clone());
        }
    }

    if options.is_empty() {
        model.popup = Some(PopupContent::Error {
            message: "No references found".to_string(),
        });
        return None;
    }

    model.select_context = Some(SelectContext::ResetIndex);
    let state = SelectPopupState::new("Reset index to".to_string(), options);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Select(state)));
    None
}

/// Pick a target tree-ish for a worktree-only reset (`git checkout <target> -- .`).
/// Includes local branches, remote branches, and tags.
/// Prioritises the first suggestion from the cursor line.
fn show_reset_worktree_picker(model: &mut Model) -> Option<Message> {
    let local_branches = get_local_branches(&model.git_info.repository);

    let remote_branches: Vec<String> = get_all_branches(&model.git_info.repository)
        .into_iter()
        .filter_map(|b| match b {
            BranchEntry::Remote(name) => Some(name),
            _ => None,
        })
        .collect();

    let tags = get_local_tags(&model.git_info.repository);

    let preferred = model
        .ui_model
        .lines
        .get(model.ui_model.cursor_position)
        .and_then(|line| suggestions_from_line(line).into_iter().next());

    let mut options: Vec<String> = Vec::new();

    if let Some(ref preferred) = preferred {
        options.push(preferred.name().to_string());
    }

    for b in &local_branches {
        if preferred.as_ref().map(|p| p.name() != b).unwrap_or(true) {
            options.push(b.clone());
        }
    }
    for b in &remote_branches {
        if preferred
            .as_ref()
            .map(|p| p.name() != b.as_str())
            .unwrap_or(true)
        {
            options.push(b.clone());
        }
    }
    for tag in &tags {
        if preferred
            .as_ref()
            .map(|p| p.name() != tag.as_str())
            .unwrap_or(true)
        {
            options.push(tag.clone());
        }
    }

    if options.is_empty() {
        model.popup = Some(PopupContent::Error {
            message: "No references found".to_string(),
        });
        return None;
    }

    model.select_context = Some(SelectContext::ResetWorktree);
    let state = SelectPopupState::new("Reset worktree to".to_string(), options);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Select(state)));
    None
}

// ── Merge ─────────────────────────────────────────────────────────────────────

/// Shows a select popup for choosing a branch to merge into the current branch.
/// The branch under the cursor (if any) is pre-selected.
fn show_merge_elsewhere(model: &mut Model) -> Option<Message> {
    let current_branch = model.git_info.current_branch();
    let mut branches: Vec<String> = get_branches(&model.git_info.repository)
        .into_iter()
        .filter(|b| current_branch.as_deref() != Some(b.as_str()))
        .collect();

    if branches.is_empty() {
        model.popup = Some(PopupContent::Error {
            message: "No branches found".to_string(),
        });
        return None;
    }

    let preferred = model
        .ui_model
        .lines
        .get(model.ui_model.cursor_position)
        .and_then(|line| {
            suggestions_from_line(line).into_iter().find(|s| match s {
                BranchSuggestion::LocalBranch(name) | BranchSuggestion::RemoteBranch(name) => {
                    current_branch.as_deref() != Some(name.as_str())
                }
                BranchSuggestion::Revision(_) => true,
            })
        });

    if let Some(ref preferred) = preferred {
        let name = preferred.name();
        if let Some(idx) = branches.iter().position(|b| b == name) {
            branches.remove(idx);
        }
        branches.insert(0, name.to_string());
    }

    model.select_context = Some(SelectContext::MergeElsewhere);
    let state = SelectPopupState::new("Merge branch".to_string(), branches);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Select(state)));
    None
}

// ── Tag ───────────────────────────────────────────────────────────────────────

/// Shows a select popup for choosing a ref/commit to tag.
/// Uses the same cursor-suggestion logic as `show_new_branch_base`.
fn show_tag_target_select(model: &mut Model, tag_name: String) -> Option<Message> {
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

    model.select_context = Some(SelectContext::CreateTagTarget(tag_name));
    let state = SelectPopupState::new("Create tag at".to_string(), options);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Select(state)));
    None
}

/// Shows a remote select popup for tag pruning.
/// If only one remote is configured, skips directly to `ShowPruneTagsConfirm`.
fn show_prune_tags_remote_pick(model: &mut Model) -> Option<Message> {
    let remotes = get_remotes(&model.git_info.repository);

    if remotes.is_empty() {
        model.popup = Some(PopupContent::Error {
            message: "No remotes configured".to_string(),
        });
        return None;
    }

    if remotes.len() == 1 {
        return Some(Message::ShowPruneTagsConfirm {
            remote: remotes.into_iter().next().unwrap(),
        });
    }

    show_remote_select(
        model,
        "Prune tags against",
        SelectContext::PruneTagsRemotePick,
    )
}

/// Shows a select popup for choosing an existing local tag to delete.
/// Pre-selects the tag under the cursor (from a Commit or LogLine ref with Tag type).
fn show_delete_tag(model: &mut Model) -> Option<Message> {
    let mut tags = get_local_tags(&model.git_info.repository);

    if tags.is_empty() {
        model.popup = Some(PopupContent::Error {
            message: "No tags found".to_string(),
        });
        return None;
    }

    // Prefer a tag from the cursor line (CommitRefType::Tag in refs)
    let preferred_tag = model
        .ui_model
        .lines
        .get(model.ui_model.cursor_position)
        .and_then(|line| match &line.content {
            LineContent::Commit(commit_info) => commit_info
                .refs
                .iter()
                .find(|r| r.ref_type == CommitRefType::Tag)
                .map(|r| r.name.clone()),
            LineContent::LogLine(entry) => entry
                .refs
                .iter()
                .find(|r| r.ref_type == CommitRefType::Tag)
                .map(|r| r.name.clone()),
            _ => None,
        });

    if let Some(ref tag) = preferred_tag
        && let Some(idx) = tags.iter().position(|t| t == tag)
    {
        let t = tags.remove(idx);
        tags.insert(0, t);
    }

    model.select_context = Some(SelectContext::DeleteTag);
    let state = SelectPopupState::new("Delete tag".to_string(), tags);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Select(state)));
    None
}

// ── Push other branch ──────────────────────────────────────────────────────────

/// Step 1: pick which local branch to push.
fn show_push_other_branch_pick(model: &mut Model) -> Option<Message> {
    let branches = get_local_branches(&model.git_info.repository);

    if branches.is_empty() {
        model.popup = Some(PopupContent::Error {
            message: "No local branches found".to_string(),
        });
        return None;
    }

    model.select_context = Some(SelectContext::PushOtherBranchPick);
    let state = SelectPopupState::new("Push branch".to_string(), branches);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Select(state)));
    None
}
