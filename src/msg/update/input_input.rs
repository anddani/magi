use crate::{
    model::{
        EditOp, Model,
        popup::{InputContext, PopupContent, PopupContentCommand},
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
            title: "Create tag at".to_string(),
            source: OptionsSource::BranchesAndTags,
            on_select: OnSelect::CreateTagTarget { name: input },
        })),
        InputContext::TagRelease { .. } => Some(Message::CreateTagRelease { name: input }),
        InputContext::Stash(_) | InputContext::RevertMainline { .. } => unreachable!(),
    }
}
