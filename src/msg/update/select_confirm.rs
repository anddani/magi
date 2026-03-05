use crate::{
    git::reset::has_uncommitted_changes,
    model::{
        LineContent, Model, ViewMode,
        popup::{
            ConfirmAction, ConfirmPopupState, PopupContent, PopupContentCommand, SelectContext,
            SelectResult,
        },
    },
    msg::{
        FetchCommand, Message, PullCommand, PushCommand, RebaseCommand, SelectPopup, StashCommand,
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
        let context = model.select_context.take();
        return route_result(context, result, model);
    }

    let result = match &model.popup {
        Some(PopupContent::Command(PopupContentCommand::Select(state))) => {
            if let Some(item) = state.selected_item() {
                // Use the selected item from filtered results
                SelectResult::Selected(item.to_string())
            } else if !state.input_text.is_empty() {
                // No matches, but user entered text - use the input text directly
                // This allows entering arbitrary values like git hashes
                SelectResult::Selected(state.input_text.clone())
            } else {
                SelectResult::NoneSelected
            }
        }
        _ => return None,
    };

    // Store the result for the caller to retrieve
    model.select_result = Some(result.clone());

    // Dismiss the popup
    model.popup = None;

    // Check context and return appropriate follow-up message
    let context = model.select_context.take();
    route_result(context, result, model)
}

fn route_result(
    context: Option<SelectContext>,
    result: SelectResult,
    model: &mut Model,
) -> Option<Message> {
    match (context, result) {
        (Some(SelectContext::CheckoutBranch), SelectResult::Selected(branch)) => {
            Some(Message::CheckoutBranch(branch))
        }
        (
            Some(SelectContext::CreateNewBranchBase { checkout }),
            SelectResult::Selected(starting_point),
        ) => Some(Message::ShowCreateNewBranchInput {
            starting_point,
            checkout,
        }),
        (Some(SelectContext::PushUpstream), SelectResult::Selected(upstream)) => {
            Some(Message::Push(PushCommand::PushToRemote(upstream)))
        }
        (Some(SelectContext::FetchUpstream), SelectResult::Selected(upstream)) => Some(
            Message::Fetch(FetchCommand::FetchFromRemoteBranch(upstream)),
        ),
        (Some(SelectContext::FetchElsewhere), SelectResult::Selected(remote)) => {
            Some(Message::Fetch(FetchCommand::FetchFromRemoteBranch(remote)))
        }
        (Some(SelectContext::PullUpstream), SelectResult::Selected(upstream)) => {
            Some(Message::Pull(PullCommand::PullFromUpstream(upstream)))
        }
        (Some(SelectContext::DeleteBranch), SelectResult::Selected(branch)) => {
            Some(Message::DeleteBranch(branch))
        }
        (Some(SelectContext::RenameBranch), SelectResult::Selected(branch)) => {
            Some(Message::ShowRenameBranchInput(branch))
        }
        (Some(SelectContext::PushAllTags), SelectResult::Selected(remote)) => {
            Some(Message::Push(PushCommand::PushAllTags(remote)))
        }
        (Some(SelectContext::PushTag), SelectResult::Selected(tag)) => {
            Some(Message::Push(PushCommand::PushTag(tag)))
        }
        (Some(SelectContext::OpenPrBranch), SelectResult::Selected(branch)) => {
            Some(Message::OpenPr {
                branch,
                target: None,
            })
        }
        (Some(SelectContext::OpenPrBranchWithTarget), SelectResult::Selected(branch)) => {
            Some(Message::ShowSelectPopup(SelectPopup::OpenPrTarget(branch)))
        }
        (Some(SelectContext::OpenPrTarget), SelectResult::Selected(target)) => {
            let branch = model.open_pr_branch.take().unwrap_or_default();
            Some(Message::OpenPr {
                branch,
                target: Some(target),
            })
        }
        (Some(SelectContext::FixupCommit(fixup_type)), SelectResult::Selected(commit)) => {
            Some(Message::FixupCommit(commit, fixup_type))
        }
        (Some(SelectContext::RebaseElsewhere), SelectResult::Selected(commit)) => {
            Some(Message::Rebase(RebaseCommand::Elsewhere(commit)))
        }
        (Some(SelectContext::WorktreeAdd { checkout }), SelectResult::Selected(branch)) => {
            Some(Message::ShowWorktreePathInput { branch, checkout })
        }
        (Some(SelectContext::ResetBranchPick), SelectResult::Selected(branch)) => Some(
            Message::ShowSelectPopup(SelectPopup::ResetBranchTarget(branch)),
        ),
        (Some(SelectContext::ResetBranchTarget(branch)), SelectResult::Selected(target)) => {
            let current_branch = model.git_info.current_branch();
            if current_branch.as_deref() == Some(branch.as_str())
                && has_uncommitted_changes(&model.git_info.repository)
            {
                model.popup = Some(PopupContent::Confirm(ConfirmPopupState {
                    message: "Uncommitted changes will be lost. Proceed?".to_string(),
                    on_confirm: ConfirmAction::ResetBranch { branch, target },
                }));
                None
            } else {
                Some(Message::ResetBranch { branch, target })
            }
        }
        (Some(SelectContext::PullPushRemote), SelectResult::Selected(remote)) => {
            Some(Message::Pull(PullCommand::PullFromPushRemote(remote)))
        }
        (Some(SelectContext::PushPushRemote), SelectResult::Selected(remote)) => {
            Some(Message::Push(PushCommand::PushToPushRemote(remote)))
        }
        (Some(SelectContext::FetchPushRemote), SelectResult::Selected(remote)) => {
            Some(Message::Fetch(FetchCommand::FetchFromPushRemote(remote)))
        }
        (Some(SelectContext::FetchAnotherBranchRemote), SelectResult::Selected(remote)) => Some(
            Message::ShowSelectPopup(SelectPopup::FetchAnotherBranchBranch(remote)),
        ),
        (Some(SelectContext::FetchAnotherBranch), SelectResult::Selected(branch)) => {
            Some(Message::Fetch(FetchCommand::FetchFromRemoteBranch(branch)))
        }
        (Some(SelectContext::ApplyStash), SelectResult::Selected(stash_display)) => {
            // Extract "stash@{N}" from "stash@{N}: message"
            let stash_ref = stash_display
                .split(": ")
                .next()
                .unwrap_or(&stash_display)
                .to_string();
            Some(Message::Stash(StashCommand::Apply(stash_ref)))
        }
        (Some(SelectContext::PopStash), SelectResult::Selected(stash_display)) => {
            // Extract "stash@{N}" from "stash@{N}: message"
            let stash_ref = stash_display
                .split(": ")
                .next()
                .unwrap_or(&stash_display)
                .to_string();
            // Show confirmation popup before popping
            model.popup = Some(PopupContent::Confirm(ConfirmPopupState {
                message: format!("Pop {}?", stash_display),
                on_confirm: ConfirmAction::PopStash(stash_ref),
            }));
            None
        }
        (Some(SelectContext::DropStash), SelectResult::Selected(stash_display)) => {
            // Extract "stash@{N}" from "stash@{N}: message"
            let stash_ref = stash_display
                .split(": ")
                .next()
                .unwrap_or(&stash_display)
                .to_string();
            // Show confirmation popup before dropping
            model.popup = Some(PopupContent::Confirm(ConfirmPopupState {
                message: format!("Drop {}?", stash_display),
                on_confirm: ConfirmAction::DropStash(stash_ref),
            }));
            None
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
    use crate::model::popup::SelectContext;
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
            select_context: None,
            pty_state: None,
            arg_mode: false,
            pending_g: false,
            arguments: None,
            open_pr_branch: None,
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
        model.select_context = Some(SelectContext::FixupCommit(FixupType::Fixup));

        let result = update(&mut model);

        assert_eq!(
            result,
            Some(Message::FixupCommit(
                "abc1234".to_string(),
                FixupType::Fixup
            ))
        );
        assert_eq!(model.view_mode, ViewMode::Status);
        assert!(model.select_context.is_none());
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
        model.select_context = Some(SelectContext::FixupCommit(FixupType::Squash));

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
        model.select_context = Some(SelectContext::RebaseElsewhere);

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
        model.select_context = Some(SelectContext::RebaseElsewhere);

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
        model.select_context = Some(SelectContext::FixupCommit(FixupType::Fixup));

        let result = update(&mut model);

        // No popup → returns None without changing state
        assert_eq!(result, None);
        // view_mode unchanged
        assert_eq!(model.view_mode, ViewMode::Log(LogType::Current, false));
        // select_context still set (not consumed)
        assert!(model.select_context.is_some());
    }
}
