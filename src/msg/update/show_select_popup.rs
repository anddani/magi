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
    i18n,
    model::{
        BranchSuggestion, LineContent, Model, Toast, ToastStyle,
        popup::{ConfirmAction, ConfirmPopupState, PopupContent, PopupContentCommand},
        select_popup::{OnSelect, OptionsSource, SelectPopupState},
        suggestions_from_line,
    },
    msg::{
        Message, PushCommand, ShowSelectPopupConfig, StashCommand, update::commit::TOAST_DURATION,
    },
};

pub fn update(model: &mut Model, config: ShowSelectPopupConfig) -> Option<Message> {
    // Special-case: stash cursor shortcuts (act immediately or show confirm)
    if let Some(msg) = handle_stash_cursor(model, &config) {
        return msg;
    }

    // Special-case: skip-if-one-remote shortcuts
    if let Some(msg) = handle_skip_if_one(model, &config) {
        return msg;
    }

    // Special-case: open PR (needs detached HEAD check)
    if let Some(msg) = handle_open_pr(model, &config) {
        return msg;
    }

    let mut options = fetch_options(model, &config.source);

    // Apply exclusion (current branch for checkout/merge, source branch for PR target)
    if let Some(ex) = compute_exclude(model, &config.on_select) {
        options.retain(|o| o != &ex);
    }

    if options.is_empty() {
        model.popup = Some(PopupContent::Error {
            message: error_msg(&config),
        });
        return None;
    }

    // Reorder/insert preferred item based on cursor or last-checked-out
    reorder_preferred(model, &mut options, &config.on_select);

    let state = SelectPopupState::new(config.title, options, config.on_select);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Select(state)));
    None
}

// ── Options fetching ──────────────────────────────────────────────────────────

fn fetch_options(model: &Model, source: &OptionsSource) -> Vec<String> {
    match source {
        OptionsSource::LocalBranches => get_local_branches(&model.git_info.repository),
        OptionsSource::LocalAndRemoteBranches => get_branches(&model.git_info.repository),
        OptionsSource::Remotes => get_remotes(&model.git_info.repository),
        OptionsSource::RemoteBranches { remote } => {
            let prefix = format!("{}/", remote);
            get_all_branches(&model.git_info.repository)
                .into_iter()
                .filter_map(|b| match b {
                    BranchEntry::Remote(name) if name.starts_with(&prefix) => Some(name),
                    _ => None,
                })
                .collect()
        }
        OptionsSource::UpstreamBranches => {
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
            get_remote_branches_for_upstream(&model.git_info.repository, suggested.as_deref())
        }
        OptionsSource::Tags => get_local_tags(&model.git_info.repository),
        OptionsSource::BranchesAndTags => {
            let mut options = get_branches(&model.git_info.repository);
            options.extend(get_local_tags(&model.git_info.repository));
            options
        }
        OptionsSource::BranchesAndTagsExcludingCheckedOut => {
            let checked_out = get_checked_out_branches(&model.workdir);
            let mut options: Vec<String> = get_branches(&model.git_info.repository)
                .into_iter()
                .filter(|b| !checked_out.contains(b.as_str()))
                .collect();
            options.extend(get_local_tags(&model.git_info.repository));
            options
        }
        OptionsSource::LocalBranchesWithRemote => get_local_branches(&model.git_info.repository)
            .into_iter()
            .filter(|b| has_any_remote(&model.workdir, b, &model.git_info.repository))
            .collect(),
        OptionsSource::FileCheckoutRevisions | OptionsSource::AllRefs => {
            let local = get_local_branches(&model.git_info.repository);
            let remote: Vec<String> = get_all_branches(&model.git_info.repository)
                .into_iter()
                .filter_map(|b| match b {
                    BranchEntry::Remote(name) => Some(name),
                    _ => None,
                })
                .collect();
            let tags = get_local_tags(&model.git_info.repository);
            let mut options = local;
            options.extend(remote);
            options.extend(tags);
            options
        }
        OptionsSource::Stashes => model
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
            .collect(),
        OptionsSource::TrackedFiles => get_tracked_files(&model.git_info.repository),
    }
}

// ── Exclusion ─────────────────────────────────────────────────────────────────

/// Returns the item to exclude from options (if any).
fn compute_exclude(model: &Model, on_select: &OnSelect) -> Option<String> {
    match on_select {
        OnSelect::CheckoutBranch | OnSelect::CheckoutLocalBranch | OnSelect::MergeElsewhere => {
            model.git_info.current_branch().map(|b| b.to_string())
        }
        OnSelect::ResetBranchTarget { branch } => Some(branch.clone()),
        OnSelect::OpenPrTarget { source_branch } => Some(source_branch.clone()),
        OnSelect::HarvestSourceBranch { .. } => {
            model.git_info.current_branch().map(|b| b.to_string())
        }
        _ => None,
    }
}

// ── Preferred item ────────────────────────────────────────────────────────────

/// Reorders `options` to place the preferred item first.
fn reorder_preferred(model: &Model, options: &mut Vec<String>, on_select: &OnSelect) {
    let preferred = compute_preferred(model, on_select);
    let insert_if_missing = should_insert_if_missing(on_select);
    if let Some(pref) = preferred.as_deref() {
        if let Some(idx) = options.iter().position(|o| o == pref) {
            let item = options.remove(idx);
            options.insert(0, item);
        } else if insert_if_missing {
            options.insert(0, pref.to_string());
        }
    }
}

/// Returns the preferred pre-selected item for the given action.
fn compute_preferred(model: &Model, on_select: &OnSelect) -> Option<String> {
    let cursor_line = model.ui_model.lines.get(model.ui_model.cursor_position);
    let current_branch = model.git_info.current_branch();

    match on_select {
        OnSelect::CheckoutBranch => {
            // Cursor branch (not current), then last checked out
            cursor_line
                .and_then(|line| {
                    suggestions_from_line(line).into_iter().find(|s| match s {
                        BranchSuggestion::LocalBranch(name) => {
                            current_branch.as_deref() != Some(name.as_str())
                        }
                        _ => true,
                    })
                })
                .map(|s| s.name().to_string())
                .or_else(|| get_last_checked_out_branch(&model.git_info.repository))
        }
        OnSelect::CheckoutLocalBranch => {
            // Cursor local branch (not current), then last checked out (in list)
            let branches = get_local_branches(&model.git_info.repository);
            cursor_line
                .and_then(|line| {
                    suggestions_from_line(line).into_iter().find(|s| {
                        matches!(s, BranchSuggestion::LocalBranch(name)
                            if current_branch.as_deref() != Some(name.as_str()))
                    })
                })
                .map(|s| s.name().to_string())
                .or_else(|| {
                    get_last_checked_out_branch(&model.git_info.repository)
                        .filter(|b| branches.contains(b))
                })
        }
        OnSelect::DeleteBranch => {
            // Cursor branch suggestion (not current local branch)
            cursor_line
                .and_then(|line| {
                    suggestions_from_line(line).into_iter().find(|s| match s {
                        BranchSuggestion::LocalBranch(name) => {
                            current_branch.as_deref() != Some(name.as_str())
                        }
                        BranchSuggestion::RemoteBranch(_) => true,
                        BranchSuggestion::Revision(_) => false,
                    })
                })
                .map(|s| s.name().to_string())
        }
        OnSelect::RenameBranch => {
            // Cursor local branch (not current), then last checked out
            cursor_line
                .and_then(|line| {
                    suggestions_from_line(line).into_iter().find(|s| {
                        matches!(s, BranchSuggestion::LocalBranch(name)
                            if current_branch.as_deref() != Some(name.as_str()))
                    })
                })
                .map(|s| s.name().to_string())
                .or_else(|| get_last_checked_out_branch(&model.git_info.repository))
        }
        OnSelect::CreateNewBranchBase { .. } | OnSelect::CreateTagTarget { .. } => {
            // Cursor suggestion (any), then current branch
            cursor_line
                .and_then(|line| suggestions_from_line(line).into_iter().next())
                .map(|s| s.name().to_string())
                .or_else(|| current_branch.map(|b| b.to_string()))
        }
        OnSelect::WorktreeAdd { .. } => {
            // Cursor suggestion (not checked out)
            let checked_out = get_checked_out_branches(&model.workdir);
            cursor_line
                .and_then(|line| {
                    suggestions_from_line(line)
                        .into_iter()
                        .find(|s| !checked_out.contains(s.name()))
                })
                .map(|s| s.name().to_string())
        }
        OnSelect::FileCheckoutRevision => {
            // Cursor suggestion (any)
            cursor_line
                .and_then(|line| suggestions_from_line(line).into_iter().next())
                .map(|s| s.name().to_string())
        }
        OnSelect::FileCheckoutFile { .. } => {
            // Cursor file line
            cursor_line.and_then(|line| match &line.content {
                LineContent::UnstagedFile(fc) => Some(fc.path.clone()),
                LineContent::StagedFile(fc) => Some(fc.path.clone()),
                LineContent::UntrackedFile(path) => Some(path.clone()),
                _ => None,
            })
        }
        OnSelect::ResetBranchPick => {
            // Cursor local branch, then current branch
            cursor_line
                .and_then(|line| {
                    suggestions_from_line(line)
                        .into_iter()
                        .find(|s| matches!(s, BranchSuggestion::LocalBranch(_)))
                })
                .map(|s| s.name().to_string())
                .or_else(|| current_branch.map(|b| b.to_string()))
        }
        OnSelect::ResetBranchTarget { .. }
        | OnSelect::Reset(_)
        | OnSelect::ResetIndex
        | OnSelect::ResetWorktree => {
            // Cursor first suggestion (any; may be a bare commit hash)
            cursor_line
                .and_then(|line| suggestions_from_line(line).into_iter().next())
                .map(|s| s.name().to_string())
        }
        OnSelect::MergeElsewhere => {
            // Cursor branch/revision (not current)
            cursor_line
                .and_then(|line| {
                    suggestions_from_line(line).into_iter().find(|s| match s {
                        BranchSuggestion::LocalBranch(name)
                        | BranchSuggestion::RemoteBranch(name) => {
                            current_branch.as_deref() != Some(name.as_str())
                        }
                        BranchSuggestion::Revision(_) => true,
                    })
                })
                .map(|s| s.name().to_string())
        }
        OnSelect::ApplyPick
        | OnSelect::ApplyApply
        | OnSelect::ApplySquash
        | OnSelect::HarvestCommitPick => {
            // Cursor commit hash (Commit or LogLine)
            cursor_line.and_then(|line| match &line.content {
                LineContent::Commit(commit_info) => Some(commit_info.hash.clone()),
                LineContent::LogLine(entry) => entry.hash.clone(),
                _ => None,
            })
        }
        OnSelect::DeleteTag => {
            // Cursor tag ref
            cursor_line.and_then(|line| match &line.content {
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
            })
        }
        OnSelect::OpenPrBranch | OnSelect::OpenPrBranchWithTarget => {
            // Cursor local branch (in filtered list), else current branch
            cursor_line
                .and_then(|line| {
                    suggestions_from_line(line)
                        .into_iter()
                        .find(|s| matches!(s, BranchSuggestion::LocalBranch(_)))
                })
                .map(|s| s.name().to_string())
                .or_else(|| current_branch.map(|b| b.to_string()))
        }
        // No preferred for remotes, upstream branches (pre-sorted), stashes, tags, etc.
        _ => None,
    }
}

/// Returns true if the preferred item should be inserted at top even when not in the options list.
fn should_insert_if_missing(on_select: &OnSelect) -> bool {
    matches!(
        on_select,
        OnSelect::CheckoutBranch           // LocalAndRemoteBranches — revision can be inserted
        | OnSelect::MergeElsewhere         // can insert revision
        | OnSelect::WorktreeAdd { .. }     // can insert non-list suggestion
        | OnSelect::CreateNewBranchBase { .. } // can insert revision/hash
        | OnSelect::FileCheckoutRevision   // can insert cursor suggestion
        | OnSelect::ResetBranchTarget { .. } // can insert cursor hash
        | OnSelect::Reset(_)               // can insert cursor hash
        | OnSelect::ResetIndex             // can insert cursor hash
        | OnSelect::ResetWorktree          // can insert cursor hash
        | OnSelect::ApplyPick              // cursor hash is inserted
        | OnSelect::ApplyApply             // cursor hash is inserted
        | OnSelect::ApplySquash            // cursor hash is inserted
        | OnSelect::HarvestCommitPick      // cursor hash is inserted
        | OnSelect::CreateTagTarget { .. } // can insert cursor suggestion
    )
}

// ── Stash cursor shortcuts ────────────────────────────────────────────────────

/// Handles stash operations when cursor is on a stash entry (acts immediately).
/// Returns `Some(msg)` when handled, `None` to fall through to the popup.
fn handle_stash_cursor(
    model: &mut Model,
    config: &ShowSelectPopupConfig,
) -> Option<Option<Message>> {
    let op = match config.on_select {
        OnSelect::ApplyStash => StashOp::Apply,
        OnSelect::PopStash => StashOp::Pop,
        OnSelect::DropStash => StashOp::Drop,
        _ => return None,
    };

    let cursor_pos = model.ui_model.cursor_position;

    // For Drop: check if cursor is on the "Stashes" section header
    if let StashOp::Drop = op
        && let Some(line) = model.ui_model.lines.get(cursor_pos)
        && let LineContent::SectionHeader { title, .. } = &line.content
        && title == i18n::t().section_stashes
    {
        model.popup = Some(PopupContent::Confirm(ConfirmPopupState {
            message: "Drop all stashes?".to_string(),
            on_confirm: ConfirmAction::DropStash("all".to_string()),
        }));
        return Some(None);
    }

    // If cursor is on a stash line, act immediately
    if let Some(line) = model.ui_model.lines.get(cursor_pos)
        && let LineContent::Stash(entry) = &line.content
    {
        let stash_ref = format!("stash@{{{}}}", entry.index);
        let msg = match op {
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
        return Some(msg);
    }

    None // Cursor not on stash, fall through to popup
}

enum StashOp {
    Apply,
    Pop,
    Drop,
}

// ── Skip-if-one-remote shortcuts ──────────────────────────────────────────────
fn with_single_remote<F>(model: &mut Model, create_msg: F) -> Option<Option<Message>>
where
    F: FnOnce(String) -> Message, // Note: Adjust `String` if your remote type differs
{
    let remotes = get_remotes(&model.git_info.repository);

    if remotes.is_empty() {
        model.popup = Some(PopupContent::error("No remotes configured".to_string()));
        return Some(None);
    }

    if remotes.len() == 1 {
        let remote = remotes.into_iter().next().unwrap();
        return Some(Some(create_msg(remote)));
    }

    None
}

/// For operations where only one remote is configured, skip the remote-selection popup
/// and go directly to the next step.
/// Returns `Some(msg)` when handled, `None` to fall through.
fn handle_skip_if_one(
    model: &mut Model,
    config: &ShowSelectPopupConfig,
) -> Option<Option<Message>> {
    match &config.on_select {
        OnSelect::PushAllTags => with_single_remote(model, |remote| {
            Message::Push(PushCommand::PushAllTags(remote))
        }),
        OnSelect::FetchAnotherBranchRemote => with_single_remote(model, |remote| {
            Message::ShowSelectPopup(ShowSelectPopupConfig {
                title: format!("Fetch branch from {}", remote),
                source: OptionsSource::RemoteBranches {
                    remote: remote.clone(),
                },
                on_select: OnSelect::FetchAnotherBranch,
            })
        }),
        OnSelect::PruneTagsRemotePick => {
            with_single_remote(model, |remote| Message::ShowPruneTagsConfirm { remote })
        }
        _ => None,
    }
}

// ── Open PR ───────────────────────────────────────────────────────────────────

/// Handles the detached-HEAD early exit for open-PR flows.
/// Returns `Some(None)` when in detached HEAD (shows toast), `None` to continue.
fn handle_open_pr(model: &mut Model, config: &ShowSelectPopupConfig) -> Option<Option<Message>> {
    if !matches!(
        config.on_select,
        OnSelect::OpenPrBranch | OnSelect::OpenPrBranchWithTarget
    ) {
        return None;
    }
    model.popup = None;
    if model.git_info.current_branch().is_none() {
        model.toast = Some(Toast {
            message: "Not checked out to a branch (detached HEAD)".to_string(),
            style: ToastStyle::Warning,
            expires_at: Instant::now() + TOAST_DURATION,
        });
        return Some(None);
    }
    None // Continue with main flow
}

// ── Error messages ────────────────────────────────────────────────────────────

fn error_msg(config: &ShowSelectPopupConfig) -> String {
    match &config.on_select {
        OnSelect::CheckoutBranch | OnSelect::MergeElsewhere | OnSelect::DeleteBranch => {
            "No branches found".to_string()
        }
        OnSelect::CheckoutLocalBranch
        | OnSelect::RenameBranch
        | OnSelect::ResetBranchPick
        | OnSelect::PushOtherBranchPick => "No local branches found".to_string(),
        OnSelect::WorktreeAdd { .. } | OnSelect::CreateNewBranchBase { .. } => {
            "No branches or tags found".to_string()
        }
        OnSelect::FileCheckoutRevision
        | OnSelect::ResetBranchTarget { .. }
        | OnSelect::Reset(_)
        | OnSelect::ResetIndex
        | OnSelect::ResetWorktree
        | OnSelect::CreateTagTarget { .. } => "No references found".to_string(),
        OnSelect::FileCheckoutFile { .. } => "No tracked files found".to_string(),
        OnSelect::ApplyStash | OnSelect::PopStash | OnSelect::DropStash => {
            "No stashes found".to_string()
        }
        OnSelect::ApplyPick
        | OnSelect::ApplyApply
        | OnSelect::ApplySquash
        | OnSelect::HarvestCommitPick => {
            "No commits or references found".to_string()
        }
        OnSelect::HarvestSourceBranch { .. } => "No local branches found".to_string(),
        OnSelect::DeleteTag => "No tags found".to_string(),
        OnSelect::PushTag => "No tags to push".to_string(),
        OnSelect::OpenPrBranch | OnSelect::OpenPrBranchWithTarget => {
            "No branches with upstream found".to_string()
        }
        OnSelect::FetchAnotherBranch => {
            if let OptionsSource::RemoteBranches { remote } = &config.source {
                format!("No branches found for remote '{}'", remote)
            } else {
                "No remote branches found".to_string()
            }
        }
        OnSelect::FetchUpstream
        | OnSelect::PullUpstream
        | OnSelect::PushUpstream
        | OnSelect::PullElsewhere
        | OnSelect::PushElsewhere
        | OnSelect::PushOtherBranchTarget { .. } => "No remote branches found".to_string(),
        _ => "No options found".to_string(),
    }
}
