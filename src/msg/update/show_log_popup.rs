use crate::{
    model::{
        Model,
        arguments::{Arguments::LogArguments, LogArgument},
        popup::{PopupContent, PopupContentCommand},
    },
    msg::Message,
};

pub fn update(model: &mut Model) -> Option<Message> {
    // Show graph and show refnames are enabled by default
    model.arguments = Some(LogArguments(
        [LogArgument::Graph, LogArgument::Decorate]
            .into_iter()
            .collect(),
    ));
    model.popup = Some(PopupContent::Command(PopupContentCommand::Log));
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Theme;
    use crate::git::GitInfo;
    use crate::git::test_repo::TestRepo;
    use crate::model::{RunningState, UiModel, ViewMode};

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
            equals_arg_mode: false,
            pending_g: false,
            arguments: None,
            view_mode: ViewMode::Status,
            cursor_reposition_context: None,
            preview_return_mode: None,
            preview_return_ui_model: None,
            log_return_ui_model: None,
            rebase_todo: None,
        }
    }

    #[test]
    fn test_graph_and_decorate_enabled_by_default() {
        let mut model = create_test_model();

        update(&mut model);

        let args = model.arguments.as_ref().and_then(|a| a.log()).unwrap();
        assert!(args.contains(&LogArgument::Graph));
        assert!(args.contains(&LogArgument::Decorate));
        assert!(!args.contains(&LogArgument::Color));
        assert!(!args.contains(&LogArgument::ShowSignature));
        assert_eq!(
            model.popup,
            Some(PopupContent::Command(PopupContentCommand::Log))
        );
    }
}
