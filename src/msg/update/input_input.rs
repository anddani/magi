use crate::{
    git::config::{set_local_config, unset_local_config},
    i18n,
    model::{
        EditOp, Model,
        popup::{ConfigurePopupState, InputContext, PopupContent, PopupContentCommand},
    },
    msg::{
        FetchCommand, Message, OnSelect, OptionsSource, PushCommand, ShowSelectPopupConfig,
        StashCommand,
    },
};

/// Handle a text editing operation in the input popup
pub fn edit(model: &mut Model, op: EditOp) -> Option<Message> {
    if let Some(PopupContent::Input(ref mut state)) = model.popup {
        state.input.apply(op);
    }
    None
}

/// Handle confirmation (Enter) in the input popup
pub fn confirm(model: &mut Model) -> Option<Message> {
    let Some(PopupContent::Input(state)) = model.popup.take() else {
        return None;
    };

    // Stash push allows empty input (git will use the default message)
    if let InputContext::Stash(stash_type) = state.context {
        return Some(Message::Stash(StashCommand::Push(
            stash_type,
            state.input.as_str().trim().to_string(),
        )));
    }

    // LogLimit: write the parsed number back into the log arguments and reopen
    // the log popup. Empty or invalid input keeps the current limit.
    if let InputContext::LogLimit = state.context {
        let input = state.input.as_str().trim();
        if let Ok(limit) = input.parse::<usize>()
            && limit > 0
            && let Some(current) = model.arguments.as_mut().and_then(|a| a.log_limit_mut())
        {
            *current = limit;
        }
        model.popup = Some(PopupContent::Command(PopupContentCommand::Log));
        model.arg_mode = false;
        return None;
    }

    // LogGrep: write the search pattern back into the log arguments and reopen
    // the log popup. Empty input clears the message filter.
    if let InputContext::LogGrep = state.context {
        let input = state.input.as_str().trim();
        if let Some(current) = model.arguments.as_mut().and_then(|a| a.log_grep_mut()) {
            *current = if input.is_empty() {
                None
            } else {
                Some(input.to_string())
            };
        }
        model.popup = Some(PopupContent::Command(PopupContentCommand::Log));
        model.arg_mode = false;
        return None;
    }

    // LogSearchChanges: write the change-search pattern back into the log
    // arguments and reopen the log popup. Empty input clears the change search.
    if let InputContext::LogSearchChanges = state.context {
        let input = state.input.as_str().trim();
        if let Some(current) = model
            .arguments
            .as_mut()
            .and_then(|a| a.log_search_changes_mut())
        {
            *current = if input.is_empty() {
                None
            } else {
                Some(input.to_string())
            };
        }
        model.popup = Some(PopupContent::Command(PopupContentCommand::Log));
        model.arg_mode = false;
        return None;
    }

    // LogSearchOccurrences: write the occurrence-search pattern back into the
    // log arguments and reopen the log popup. Empty input clears it.
    if let InputContext::LogSearchOccurrences = state.context {
        let input = state.input.as_str().trim();
        if let Some(current) = model
            .arguments
            .as_mut()
            .and_then(|a| a.log_search_occurrences_mut())
        {
            *current = if input.is_empty() {
                None
            } else {
                Some(input.to_string())
            };
        }
        model.popup = Some(PopupContent::Command(PopupContentCommand::Log));
        model.arg_mode = false;
        return None;
    }

    // LogTraceLines: write the line-trace range back into the log arguments and
    // reopen the log popup. Empty input clears it.
    if let InputContext::LogTraceLines = state.context {
        let input = state.input.as_str().trim();
        if let Some(current) = model
            .arguments
            .as_mut()
            .and_then(|a| a.log_trace_lines_mut())
        {
            *current = if input.is_empty() {
                None
            } else {
                Some(input.to_string())
            };
        }
        model.popup = Some(PopupContent::Command(PopupContentCommand::Log));
        model.arg_mode = false;
        return None;
    }

    // LogFiles: write the pathspec file filter back into the log arguments and
    // reopen the log popup. Empty input clears it.
    if let InputContext::LogFiles = state.context {
        let input = state.input.as_str().trim();
        if let Some(current) = model.arguments.as_mut().and_then(|a| a.log_files_mut()) {
            *current = if input.is_empty() {
                None
            } else {
                Some(input.to_string())
            };
        }
        model.popup = Some(PopupContent::Command(PopupContentCommand::Log));
        model.arg_mode = false;
        return None;
    }

    // ConfigDescription: set or unset `branch.<b>.description`, then reopen the
    // Configure popup. Empty input clears the description.
    if let InputContext::ConfigDescription { branch } = state.context {
        let input = state.input.as_str().trim();
        let key = format!("branch.{}.description", branch);
        if input.is_empty() {
            let _ = unset_local_config(&model.workdir, &key);
        } else {
            let _ = set_local_config(&model.workdir, &key, input);
        }
        model.popup = Some(PopupContent::Command(PopupContentCommand::Configure(
            ConfigurePopupState {
                branch,
                pending_a: false,
            },
        )));
        return None;
    }

    // RevertMainline allows empty input (empty = clear the mainline value)
    if let InputContext::RevertMainline { mut revert_state } = state.context {
        let input = state.input.as_str().trim().to_string();
        revert_state.mainline = if input.is_empty() { None } else { Some(input) };
        model.popup = Some(PopupContent::Command(PopupContentCommand::Revert(
            revert_state,
        )));
        model.arg_mode = false;
        return None;
    }

    let input = state.input.as_str().trim().to_string();
    if input.is_empty() {
        // Restore the popup if input is empty
        model.popup = Some(PopupContent::Input(state));
        return None;
    }

    match state.context {
        InputContext::CreateNewBranch {
            starting_point,
            checkout,
        } => Some(Message::CreateNewBranch {
            starting_point,
            branch_name: input,
            checkout,
        }),
        InputContext::RenameBranch { old_name } => Some(Message::RenameBranch {
            old_name,
            new_name: input,
        }),
        InputContext::SpinoffBranch => Some(Message::SpinoffBranch(input)),
        InputContext::SpinoutBranch => Some(Message::SpinoutBranch(input)),
        InputContext::CherrySpinout { commits, root } => Some(Message::CherrySpinout {
            commits,
            branch: input,
            root,
        }),
        InputContext::CherrySpinoff { commits, root } => Some(Message::CherrySpinoff {
            commits,
            branch: input,
            root,
        }),
        InputContext::WorktreePath { branch, checkout } => Some(Message::WorktreeCheckout {
            branch,
            path: input,
            checkout,
        }),
        InputContext::WorktreeBranchName { starting_point } => {
            Some(Message::ShowWorktreeBranchPathInput {
                starting_point,
                branch_name: input,
            })
        }
        InputContext::WorktreeBranchPath {
            starting_point,
            branch_name,
        } => Some(Message::WorktreeBranch {
            starting_point,
            branch_name,
            path: input,
        }),
        InputContext::WorktreeMovePath { worktree } => Some(Message::WorktreeMove {
            worktree,
            path: input,
        }),
        InputContext::PushRefspec { remote } => Some(Message::Push(PushCommand::PushRefspecs {
            remote,
            refspecs: input,
        })),
        InputContext::FetchRefspec { remote } => {
            Some(Message::Fetch(FetchCommand::FetchRefspecs {
                remote,
                refspecs: input,
            }))
        }
        InputContext::CreateTag => Some(Message::ShowSelectPopup(ShowSelectPopupConfig {
            title: i18n::t().title_create_tag_at.to_string(),
            source: OptionsSource::BranchesAndTags,
            on_select: OnSelect::CreateTagTarget { name: input },
        })),
        InputContext::TagRelease { .. } => Some(Message::CreateTagRelease { name: input }),
        InputContext::StashBranch { stash_ref } => Some(Message::Stash(StashCommand::Branch {
            stash_ref,
            branch_name: input,
        })),
        InputContext::StashBranchHere { stash_ref } => {
            Some(Message::Stash(StashCommand::BranchHere {
                stash_ref,
                branch_name: input,
            }))
        }
        InputContext::Stash(_)
        | InputContext::RevertMainline { .. }
        | InputContext::LogLimit
        | InputContext::LogGrep
        | InputContext::LogSearchChanges
        | InputContext::LogSearchOccurrences
        | InputContext::LogTraceLines
        | InputContext::LogFiles
        | InputContext::ConfigDescription { .. } => {
            unreachable!()
        }
    }
}
