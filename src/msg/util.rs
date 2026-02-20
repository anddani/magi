use std::collections::HashSet;

use crate::{
    model::{Line, SectionType},
    msg::Message,
};

/// Count visible lines between two raw line indices (exclusive of end).
/// Used to calculate scroll offsets in terms of visible lines.
pub fn visible_lines_between(
    lines: &[Line],
    start: usize,
    end: usize,
    collapsed: &HashSet<SectionType>,
) -> usize {
    lines
        .iter()
        .skip(start)
        .take(end.saturating_sub(start))
        .filter(|line| !line.is_hidden(collapsed))
        .count()
}

/// Returns true if [`Message`] requires to pause Ratatui rendering
pub fn is_external_command(msg: &Message) -> bool {
    match msg {
        Message::Commit | Message::Amend(_) | Message::FixupCommit(_, _) => true,
        Message::Quit
        | Message::Refresh
        | Message::MoveUp
        | Message::MoveDown
        | Message::HalfPageUp
        | Message::HalfPageDown
        | Message::ScrollLineDown
        | Message::ScrollLineUp
        | Message::ToggleSection
        | Message::StageAllModified
        | Message::StageSelected
        | Message::UnstageSelected
        | Message::UnstageAll
        | Message::DiscardSelected
        | Message::ConfirmDiscard(_)
        | Message::DismissPopup
        | Message::EnterVisualMode
        | Message::ExitVisualMode
        | Message::ShowPopup(_)
        | Message::ShowPushPopup
        | Message::ShowFetchPopup
        | Message::ShowCheckoutBranchPopup
        | Message::ShowCheckoutLocalBranchPopup
        | Message::CheckoutBranch(_)
        | Message::FetchAllRemotes
        | Message::FetchUpstream
        | Message::ShowFetchUpstreamSelect
        | Message::ShowFetchElsewhereSelect
        | Message::FetchFromRemote(_)
        | Message::PushUpstream
        | Message::ShowPushUpstreamSelect
        | Message::PushToRemote(_)
        | Message::ShowPushAllTagsSelect
        | Message::PushAllTags(_)
        | Message::ShowPushTagSelect
        | Message::PushTag(_)
        | Message::ShowPullPopup
        | Message::PullUpstream
        | Message::ShowPullUpstreamSelect
        | Message::PullFromRemote(_)
        | Message::EnterArgMode
        | Message::ToggleArgument(_)
        | Message::ExitArgMode
        | Message::Select(_)
        | Message::Credentials(_)
        | Message::MoveToTop
        | Message::MoveToBottom
        | Message::PendingG
        | Message::ShowDeleteBranchPopup
        | Message::ShowRenameBranchPopup
        | Message::ShowRenameBranchInput(_)
        | Message::RenameBranch { .. }
        | Message::DeleteBranch(_)
        | Message::ConfirmDeleteBranch(_)
        | Message::ShowCreateNewBranchPopup { .. }
        | Message::ShowCreateNewBranchInput { .. }
        | Message::CreateNewBranch { .. }
        | Message::Input(_)
        | Message::ShowOpenPrSelect
        | Message::ShowOpenPrWithTargetSelect
        | Message::ShowOpenPrTargetSelect(_)
        | Message::OpenPr { .. }
        | Message::ShowLog(_)
        | Message::ExitLogView
        | Message::ShowFixupCommitSelect(_) => false,
    }
}
