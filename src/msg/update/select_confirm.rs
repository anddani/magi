use crate::{
    model::{
        Model,
        popup::{PopupContent, PopupContentCommand, SelectContext, SelectResult},
    },
    msg::Message,
};

pub fn update(model: &mut Model) -> Option<Message> {
    let result =
        if let Some(PopupContent::Command(PopupContentCommand::Select(ref state))) = model.popup {
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
        } else {
            return None;
        };

    // Store the result for the caller to retrieve
    model.select_result = Some(result.clone());

    // Dismiss the popup
    model.popup = None;

    // Check context and return appropriate follow-up message
    let context = model.select_context.take();
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
            Some(Message::PushToRemote(upstream))
        }
        (Some(SelectContext::FetchUpstream), SelectResult::Selected(upstream)) => {
            Some(Message::FetchFromRemote(upstream))
        }
        (Some(SelectContext::FetchElsewhere), SelectResult::Selected(remote)) => {
            Some(Message::FetchFromRemote(remote))
        }
        (Some(SelectContext::PullUpstream), SelectResult::Selected(upstream)) => {
            Some(Message::PullFromRemote(upstream))
        }
        (Some(SelectContext::DeleteBranch), SelectResult::Selected(branch)) => {
            Some(Message::DeleteBranch(branch))
        }
        (Some(SelectContext::RenameBranch), SelectResult::Selected(branch)) => {
            Some(Message::ShowRenameBranchInput(branch))
        }
        (Some(SelectContext::PushAllTags), SelectResult::Selected(remote)) => {
            Some(Message::PushAllTags(remote))
        }
        (Some(SelectContext::PushTag), SelectResult::Selected(tag)) => Some(Message::PushTag(tag)),
        (Some(SelectContext::OpenPrBranch), SelectResult::Selected(branch)) => {
            Some(Message::OpenPr {
                branch,
                target: None,
            })
        }
        (Some(SelectContext::OpenPrBranchWithTarget), SelectResult::Selected(branch)) => {
            Some(Message::ShowOpenPrTargetSelect(branch))
        }
        (Some(SelectContext::OpenPrTarget), SelectResult::Selected(target)) => {
            let branch = model.open_pr_branch.take().unwrap_or_default();
            Some(Message::OpenPr {
                branch,
                target: Some(target),
            })
        }
        (Some(SelectContext::FixupCommit), SelectResult::Selected(commit)) => {
            Some(Message::FixupCommit(commit))
        }
        _ => None,
    }
}
