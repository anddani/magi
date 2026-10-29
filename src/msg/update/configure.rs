use crate::{
    git::{
        config::{
            cycle_config_value, get_local_config, set_local_config, unset_branch_upstream,
            unset_local_config,
        },
        push::get_remotes,
    },
    i18n,
    model::{
        Model,
        popup::{InputContext, PopupContent, PopupContentCommand},
        select_popup::{OnSelect, OptionsSource},
    },
    msg::{ConfigAction, Message, ShowSelectPopupConfig},
};

/// Handle an action taken inside the Configure popup. Reads the scoped branch
/// from the popup state, writes git config immediately, and leaves the popup in
/// place so it re-renders from the live config.
pub fn update(model: &mut Model, action: ConfigAction) -> Option<Message> {
    // The scoped branch lives in the popup state.
    let branch = match &model.popup {
        Some(PopupContent::Command(PopupContentCommand::Configure(state))) => state.branch.clone(),
        _ => return None,
    };

    match action {
        ConfigAction::SetPendingA => {
            set_pending_a(model, true);
            return None;
        }
        ConfigAction::ClearPendingA => {
            set_pending_a(model, false);
            return None;
        }
        ConfigAction::EditDescription => {
            let current =
                get_local_config(&model.workdir, &format!("branch.{}.description", branch))
                    .unwrap_or_default();
            model.popup = Some(PopupContent::Input(
                crate::model::popup::InputPopupState::with_text(
                    InputContext::ConfigDescription { branch },
                    current,
                ),
            ));
            return None;
        }
        ConfigAction::ToggleUpstream => {
            let has_upstream =
                get_local_config(&model.workdir, &format!("branch.{}.remote", branch)).is_some()
                    || get_local_config(&model.workdir, &format!("branch.{}.merge", branch))
                        .is_some();
            if has_upstream {
                let _ = unset_branch_upstream(&model.workdir, &branch);
                set_pending_a(model, false);
                return None;
            }
            return Some(Message::ShowSelectPopup(ShowSelectPopupConfig {
                title: i18n::t().fmt1(i18n::t().title_configure_upstream_fmt, &branch),
                source: OptionsSource::UpstreamBranches,
                on_select: OnSelect::ConfigSetUpstream { branch },
            }));
        }
        ConfigAction::CycleBranchRebase => {
            cycle(
                model,
                &format!("branch.{}.rebase", branch),
                &["true", "false"],
            );
        }
        ConfigAction::CyclePullRebase => {
            cycle(model, "pull.rebase", &["true", "false"]);
        }
        ConfigAction::CycleAutoSetupMerge => {
            cycle(model, "branch.autoSetupMerge", &["always", "true", "false"]);
        }
        ConfigAction::CycleAutoSetupRebase => {
            cycle(
                model,
                "branch.autoSetupRebase",
                &["always", "local", "remote", "never"],
            );
        }
        ConfigAction::CycleBranchPushRemote => {
            let remotes = get_remotes(&model.git_info.repository);
            let choices: Vec<&str> = remotes.iter().map(|s| s.as_str()).collect();
            cycle(model, &format!("branch.{}.pushRemote", branch), &choices);
        }
        ConfigAction::CyclePushDefault => {
            let remotes = get_remotes(&model.git_info.repository);
            let choices: Vec<&str> = remotes.iter().map(|s| s.as_str()).collect();
            cycle(model, "remote.pushDefault", &choices);
        }
    }

    // Any action that fell through (a cycle) clears the `a` prefix and keeps the
    // Configure popup open so it re-renders from the freshly written config.
    set_pending_a(model, false);
    None
}

/// Reads `key`, computes the next value in `choices`, and writes it back
/// (unsetting when the cycle wraps past the last choice).
fn cycle(model: &Model, key: &str, choices: &[&str]) {
    let current = get_local_config(&model.workdir, key);
    match cycle_config_value(current.as_deref(), choices) {
        Some(value) => {
            let _ = set_local_config(&model.workdir, key, &value);
        }
        None => {
            let _ = unset_local_config(&model.workdir, key);
        }
    }
}

/// Sets the Configure popup's `pending_a` flag, if the popup is open.
fn set_pending_a(model: &mut Model, value: bool) {
    if let Some(PopupContent::Command(PopupContentCommand::Configure(state))) = model.popup.as_mut()
    {
        state.pending_a = value;
    }
}
