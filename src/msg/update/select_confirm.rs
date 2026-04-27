use crate::{
    git::reset::has_uncommitted_changes,
    model::{
        LineContent, Model, ViewMode,
        popup::{
            ConfirmAction, ConfirmPopupState, PopupContent, PopupContentCommand, SelectResult,
        },
        select_popup::OnSelect,
    },
    msg::{
        ApplyCommand, FetchCommand, MergeCommand, Message, OptionsSource, PullCommand, PushCommand,
        RebaseCommand, ResetMode, RevertCommand, ShowSelectPopupConfig, StashCommand,
    },
};

pub fn update(model: &mut Model) -> Option<Message> {
    // Handle log pick mode: Enter extracts hash from cursor line
    if let ViewMode::Log(_, true) = model.view_mode {
        let hash = model
            .ui_model
            .lines
            .get(model.ui_model.cursor_position)
            .and_then(|line| match &line.content {
                LineContent::LogLine(entry) => entry.hash.as_deref(),
                _ => None,
            });
        let result = match hash {
            Some(h) => SelectResult::Selected(h.to_string()),
            None => SelectResult::NoneSelected,
        };
        model.view_mode = ViewMode::Status;
        model.select_result = Some(result.clone());
        let on_select = model.log_pick_on_select.take();
        return route_result(on_select, result, model);
    }

    // Extract result and on_select from the select popup state
    let (on_select, result) = match model.popup.take() {
        Some(PopupContent::Command(PopupContentCommand::Select(state))) => {
            let result = if let Some(item) = state.selected_item() {
                SelectResult::Selected(item.to_string())
            } else if !state.input_text.is_empty() {
                // No matches but user typed text — use it directly (allows git hashes)
                SelectResult::Selected(state.input_text.clone())
            } else {
                SelectResult::NoneSelected
            };
            (state.on_select, result)
        }
        other => {
            model.popup = other;
            return None;
        }
    };

    model.select_result = Some(result.clone());
    route_result(Some(on_select), result, model)
}

fn route_result(
    on_select: Option<OnSelect>,
    result: SelectResult,
    model: &mut Model,
) -> Option<Message> {
    match (on_select, result) {
        (
            Some(OnSelect::CheckoutBranch) | Some(OnSelect::CheckoutLocalBranch),
            SelectResult::Selected(branch),
        ) => Some(Message::CheckoutBranch(branch)),
        (
            Some(OnSelect::CreateNewBranchBase { checkout }),
            SelectResult::Selected(starting_point),
        ) => Some(Message::ShowCreateNewBranchInput {
            starting_point,
            checkout,
        }),
        (Some(OnSelect::PushUpstream), SelectResult::Selected(upstream)) => {
            Some(Message::Push(PushCommand::PushToRemote(upstream)))
        }
        (Some(OnSelect::FetchUpstream), SelectResult::Selected(upstream)) => Some(Message::Fetch(
            FetchCommand::FetchFromRemoteBranch(upstream),
        )),
        (Some(OnSelect::FetchElsewhere), SelectResult::Selected(remote)) => {
            Some(Message::Fetch(FetchCommand::FetchFromRemoteBranch(remote)))
        }
        (Some(OnSelect::PullUpstream), SelectResult::Selected(upstream)) => {
            Some(Message::Pull(PullCommand::PullFromUpstream(upstream)))
        }
        (Some(OnSelect::DeleteBranch), SelectResult::Selected(branch)) => {
            Some(Message::DeleteBranch(branch))
        }
        (Some(OnSelect::RenameBranch), SelectResult::Selected(branch)) => {
            Some(Message::ShowRenameBranchInput(branch))
        }
        (Some(OnSelect::PushAllTags), SelectResult::Selected(remote)) => {
            Some(Message::Push(PushCommand::PushAllTags(remote)))
        }
        (Some(OnSelect::PushTag), SelectResult::Selected(tag)) => {
            Some(Message::Push(PushCommand::PushTag(tag)))
        }
        (Some(OnSelect::OpenPrBranch), SelectResult::Selected(branch)) => Some(Message::OpenPr {
            branch,
            target: None,
        }),
        (Some(OnSelect::OpenPrBranchWithTarget), SelectResult::Selected(branch)) => {
            Some(Message::ShowSelectPopup(ShowSelectPopupConfig {
                title: "Open PR to".to_string(),
                source: OptionsSource::LocalBranches,
                on_select: OnSelect::OpenPrTarget {
                    source_branch: branch,
                },
            }))
        }
        (Some(OnSelect::OpenPrTarget { source_branch }), SelectResult::Selected(target)) => {
            Some(Message::OpenPr {
                branch: source_branch,
                target: Some(target),
            })
        }
        (Some(OnSelect::FixupCommit(fixup_type)), SelectResult::Selected(commit)) => {
            Some(Message::FixupCommit(commit, fixup_type))
        }
        (Some(OnSelect::RebaseElsewhere), SelectResult::Selected(commit)) => {
            Some(Message::Rebase(RebaseCommand::Elsewhere(commit)))
        }
        (Some(OnSelect::ReviseCommit), SelectResult::Selected(hash)) => {
            Some(Message::ReviseCommit(hash))
        }
        (Some(OnSelect::WorktreeAdd { checkout }), SelectResult::Selected(branch)) => {
            Some(Message::ShowWorktreePathInput { branch, checkout })
        }
        (Some(OnSelect::ResetBranchPick), SelectResult::Selected(branch)) => {
            Some(Message::ShowSelectPopup(ShowSelectPopupConfig {
                title: "Reset branch to".to_string(),
                source: OptionsSource::AllRefs,
                on_select: OnSelect::ResetBranchTarget { branch },
            }))
        }
        (Some(OnSelect::FileCheckoutRevision), SelectResult::Selected(revision)) => {
            Some(Message::ShowSelectPopup(ShowSelectPopupConfig {
                title: "File to checkout".to_string(),
                source: OptionsSource::TrackedFiles,
                on_select: OnSelect::FileCheckoutFile { revision },
            }))
        }
        (Some(OnSelect::FileCheckoutFile { revision }), SelectResult::Selected(file)) => {
            Some(Message::FileCheckout { revision, file })
        }
        (Some(OnSelect::ResetBranchTarget { branch }), SelectResult::Selected(target)) => {
            let current_branch = model.git_info.current_branch();
            if current_branch.as_deref() == Some(branch.as_str())
                && has_uncommitted_changes(&model.git_info.repository)
            {
                model.popup = Some(PopupContent::Confirm(ConfirmPopupState {
                    message: "Uncommitted changes will be lost. Proceed?".to_string(),
                    on_confirm: ConfirmAction::ResetBranch {
                        branch,
                        target,
                        mode: ResetMode::Hard,
                    },
                }));
                None
            } else {
                Some(Message::ResetBranch {
                    branch,
                    target,
                    mode: ResetMode::Hard,
                })
            }
        }
        (Some(OnSelect::Reset(mode)), SelectResult::Selected(target)) => {
            let branch = model.git_info.current_branch().unwrap_or_default();
            Some(Message::ResetBranch {
                branch,
                target,
                mode,
            })
        }
        (Some(OnSelect::ResetIndex), SelectResult::Selected(target)) => {
            Some(Message::ResetIndex { target })
        }
        (Some(OnSelect::ResetWorktree), SelectResult::Selected(target)) => {
            Some(Message::ResetWorktree { target })
        }
        (Some(OnSelect::PullPushRemote), SelectResult::Selected(remote)) => {
            Some(Message::Pull(PullCommand::PullFromPushRemote(remote)))
        }
        (Some(OnSelect::PullElsewhere), SelectResult::Selected(upstream)) => {
            Some(Message::Pull(PullCommand::PullFromElsewhere(upstream)))
        }
        (Some(OnSelect::PushElsewhere), SelectResult::Selected(upstream)) => {
            Some(Message::Push(PushCommand::PushElsewhere(upstream)))
        }
        (Some(OnSelect::PushOtherBranchPick), SelectResult::Selected(branch)) => {
            Some(Message::ShowSelectPopup(ShowSelectPopupConfig {
                title: "Push to".to_string(),
                source: OptionsSource::UpstreamBranches,
                on_select: OnSelect::PushOtherBranchTarget { local: branch },
            }))
        }
        (Some(OnSelect::PushOtherBranchTarget { local }), SelectResult::Selected(remote)) => {
            Some(Message::Push(PushCommand::PushOtherBranch {
                local,
                remote,
            }))
        }
        (Some(OnSelect::PushRefspecRemotePick), SelectResult::Selected(remote)) => {
            Some(Message::ShowPushRefspecInput(remote))
        }
        (Some(OnSelect::FetchRefspecRemotePick), SelectResult::Selected(remote)) => {
            Some(Message::ShowFetchRefspecInput(remote))
        }
        (Some(OnSelect::PushMatching), SelectResult::Selected(remote)) => {
            Some(Message::Push(PushCommand::PushMatching(remote)))
        }
        (Some(OnSelect::PushPushRemote), SelectResult::Selected(remote)) => {
            Some(Message::Push(PushCommand::PushToPushRemote(remote)))
        }
        (Some(OnSelect::FetchPushRemote), SelectResult::Selected(remote)) => {
            Some(Message::Fetch(FetchCommand::FetchFromPushRemote(remote)))
        }
        (Some(OnSelect::FetchAnotherBranchRemote), SelectResult::Selected(remote)) => {
            Some(Message::ShowSelectPopup(ShowSelectPopupConfig {
                title: format!("Fetch branch from {}", remote),
                source: OptionsSource::RemoteBranches {
                    remote: remote.clone(),
                },
                on_select: OnSelect::FetchAnotherBranch,
            }))
        }
        (Some(OnSelect::FetchAnotherBranch), SelectResult::Selected(branch)) => {
            Some(Message::Fetch(FetchCommand::FetchFromRemoteBranch(branch)))
        }
        (Some(OnSelect::ApplyStash), SelectResult::Selected(stash_display)) => {
            let stash_ref = stash_display
                .split(": ")
                .next()
                .unwrap_or(&stash_display)
                .to_string();
            Some(Message::Stash(StashCommand::Apply(stash_ref)))
        }
        (Some(OnSelect::PopStash), SelectResult::Selected(stash_display)) => {
            let stash_ref = stash_display
                .split(": ")
                .next()
                .unwrap_or(&stash_display)
                .to_string();
            model.popup = Some(PopupContent::Confirm(ConfirmPopupState {
                message: format!("Pop {}?", stash_display),
                on_confirm: ConfirmAction::PopStash(stash_ref),
            }));
            None
        }
        (Some(OnSelect::DropStash), SelectResult::Selected(stash_display)) => {
            let stash_ref = stash_display
                .split(": ")
                .next()
                .unwrap_or(&stash_display)
                .to_string();
            model.popup = Some(PopupContent::Confirm(ConfirmPopupState {
                message: format!("Drop {}?", stash_display),
                on_confirm: ConfirmAction::DropStash(stash_ref),
            }));
            None
        }
        (Some(OnSelect::MergeElsewhere), SelectResult::Selected(branch)) => {
            Some(Message::Merge(MergeCommand::Branch(branch)))
        }
        (Some(OnSelect::ApplyPick), SelectResult::Selected(hash)) => {
            Some(Message::Apply(ApplyCommand::Pick(vec![hash])))
        }
        (Some(OnSelect::ApplyApply), SelectResult::Selected(hash)) => {
            Some(Message::Apply(ApplyCommand::Apply(vec![hash])))
        }
        (Some(OnSelect::ApplySquash), SelectResult::Selected(hash)) => {
            Some(Message::Apply(ApplyCommand::Squash(hash)))
        }
        (Some(OnSelect::DonateCommitPick), SelectResult::Selected(hash)) => {
            Some(Message::ShowSelectPopup(ShowSelectPopupConfig {
                title: "Donate to branch".to_string(),
                source: OptionsSource::LocalBranches,
                on_select: OnSelect::DonateTargetBranch {
                    commits: vec![hash],
                },
            }))
        }
        (Some(OnSelect::DonateTargetBranch { commits }), SelectResult::Selected(target)) => {
            Some(Message::Donate { commits, target })
        }
        (Some(OnSelect::HarvestCommitPick), SelectResult::Selected(hash)) => {
            Some(Message::ShowSelectPopup(ShowSelectPopupConfig {
                title: "Harvest from branch".to_string(),
                source: OptionsSource::LocalBranches,
                on_select: OnSelect::HarvestSourceBranch {
                    commits: vec![hash],
                },
            }))
        }
        (Some(OnSelect::HarvestSourceBranch { commits }), SelectResult::Selected(source)) => {
            Some(Message::Harvest { commits, source })
        }
        (Some(OnSelect::CreateTagTarget { name }), SelectResult::Selected(target)) => {
            Some(Message::CreateTag { name, target })
        }
        (Some(OnSelect::DeleteTag), SelectResult::Selected(tag)) => Some(Message::DeleteTag(tag)),
        (Some(OnSelect::PruneTagsRemotePick), SelectResult::Selected(remote)) => {
            Some(Message::ShowPruneTagsConfirm { remote })
        }
        (
            Some(OnSelect::RevertMergeMainline { hashes, no_commit }),
            SelectResult::Selected(selection),
        ) => {
            let mainline = selection
                .chars()
                .next()
                .and_then(|c| c.to_digit(10))
                .unwrap_or(1) as u8;
            Some(Message::Revert(RevertCommand::CommitsWithMainline {
                hashes,
                mainline,
                no_commit,
            }))
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Theme;
    use crate::git::GitInfo;
    use crate::git::test_repo::TestRepo;
    use crate::model::log_view::LogEntry;
    use crate::model::select_popup::OnSelect;
    use crate::model::{Line, LineContent, RunningState, UiModel, ViewMode};
    use crate::msg::{FixupType, LogType};

    fn create_test_model() -> Model {
        let test_repo = TestRepo::new();
        let repo_path = test_repo.repo.workdir().unwrap();
        let git_info = GitInfo::new_from_path(repo_path).unwrap();
        let workdir = repo_path.to_path_buf();
        Model {
            git_info,
            workdir,
            running_state: RunningState::Running,
            ui_model: UiModel::default(),
            theme: Theme::default(),
            popup: None,
            toast: None,
            select_result: None,
            log_pick_on_select: None,
            pty_state: None,
            arg_mode: false,
            pending_g: false,
            arguments: None,
            view_mode: ViewMode::Status,
            cursor_reposition_context: None,
            preview_return_mode: None,
            preview_return_ui_model: None,
        }
    }

    fn make_log_line(hash: &str, message: &str) -> Line {
        Line {
            content: LineContent::LogLine(LogEntry {
                hash: Some(hash.to_string()),
                message: Some(message.to_string()),
                author: None,
                time: None,
                refs: vec![],
                graph: String::new(),
            }),
            section: None,
        }
    }

    #[test]
    fn test_log_pick_mode_extracts_hash_from_cursor() {
        let mut model = create_test_model();
        model.view_mode = ViewMode::Log(LogType::Current, true);
        model.ui_model.lines = vec![
            make_log_line("abc1234", "First commit"),
            make_log_line("def5678", "Second commit"),
        ];
        model.ui_model.cursor_position = 0;
        model.log_pick_on_select = Some(OnSelect::FixupCommit(FixupType::Fixup));

        let result = update(&mut model);

        assert_eq!(
            result,
            Some(Message::FixupCommit(
                "abc1234".to_string(),
                FixupType::Fixup
            ))
        );
        assert_eq!(model.view_mode, ViewMode::Status);
        assert!(model.log_pick_on_select.is_none());
    }

    #[test]
    fn test_log_pick_mode_second_cursor_position() {
        let mut model = create_test_model();
        model.view_mode = ViewMode::Log(LogType::Current, true);
        model.ui_model.lines = vec![
            make_log_line("abc1234", "First commit"),
            make_log_line("def5678", "Second commit"),
        ];
        model.ui_model.cursor_position = 1;
        model.log_pick_on_select = Some(OnSelect::FixupCommit(FixupType::Squash));

        let result = update(&mut model);

        assert_eq!(
            result,
            Some(Message::FixupCommit(
                "def5678".to_string(),
                FixupType::Squash
            ))
        );
        assert_eq!(model.view_mode, ViewMode::Status);
    }

    #[test]
    fn test_log_pick_mode_no_hash_returns_none() {
        let mut model = create_test_model();
        model.view_mode = ViewMode::Log(LogType::Current, true);
        model.ui_model.lines = vec![Line {
            content: LineContent::LogLine(LogEntry {
                hash: None,
                message: Some("Graph only line".to_string()),
                author: None,
                time: None,
                refs: vec![],
                graph: String::new(),
            }),
            section: None,
        }];
        model.ui_model.cursor_position = 0;
        model.log_pick_on_select = Some(OnSelect::RebaseElsewhere);

        let result = update(&mut model);

        // NoneSelected context → None message
        assert_eq!(result, None);
        assert_eq!(model.view_mode, ViewMode::Status);
    }

    #[test]
    fn test_log_pick_mode_routes_rebase_elsewhere() {
        use crate::msg::RebaseCommand;

        let mut model = create_test_model();
        model.view_mode = ViewMode::Log(LogType::AllReferences, true);
        model.ui_model.lines = vec![make_log_line("deadbeef", "Some commit")];
        model.ui_model.cursor_position = 0;
        model.log_pick_on_select = Some(OnSelect::RebaseElsewhere);

        let result = update(&mut model);

        assert_eq!(
            result,
            Some(Message::Rebase(RebaseCommand::Elsewhere(
                "deadbeef".to_string()
            )))
        );
        assert_eq!(model.view_mode, ViewMode::Status);
    }

    #[test]
    fn test_browse_log_mode_does_not_trigger_pick() {
        let mut model = create_test_model();
        // picking = false → should fall through to popup check and return None (no popup)
        model.view_mode = ViewMode::Log(LogType::Current, false);
        model.ui_model.lines = vec![make_log_line("abc1234", "First commit")];
        model.log_pick_on_select = Some(OnSelect::FixupCommit(FixupType::Fixup));

        let result = update(&mut model);

        // No popup → returns None without changing state
        assert_eq!(result, None);
        // view_mode unchanged
        assert_eq!(model.view_mode, ViewMode::Log(LogType::Current, false));
        // log_pick_on_select still set (not consumed)
        assert!(model.log_pick_on_select.is_some());
    }
}
