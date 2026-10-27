use crate::{
    git::git_cmd,
    i18n,
    model::{Model, popup::PopupContent},
    msg::Message,
};

pub fn update(model: &mut Model, commits: Vec<String>, target: String) -> Option<Message> {
    model.popup = None;

    if commits.is_empty() {
        return None;
    }

    // Remember the current branch so we can return after donating
    let current_branch: Option<String> =
        git_cmd(&model.workdir, &["rev-parse", "--abbrev-ref", "HEAD"])
            .output()
            .ok()
            .filter(|o| o.status.success())
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
            .filter(|s| s != "HEAD"); // ignore detached HEAD

    // Step 1: checkout the target branch
    let checkout_output = git_cmd(&model.workdir, &["checkout", target.as_str()]).output();

    match checkout_output {
        Err(e) => {
            model.popup = Some(PopupContent::Error {
                message: format!("{}: {}", i18n::t().err_checkout_target_branch_failed, e),
            });
            return Some(Message::Refresh);
        }
        Ok(output) if !output.status.success() => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            model.popup = Some(PopupContent::Error {
                message: format!(
                    "{}: {}",
                    i18n::t().fmt1(i18n::t().err_checkout_named_failed_fmt, &target),
                    stderr.trim()
                ),
            });
            return Some(Message::Refresh);
        }
        Ok(_) => {}
    }

    // Step 2: cherry-pick the commits onto the target branch
    let mut cherry_pick_args: Vec<&str> = vec!["cherry-pick"];
    let commits_refs: Vec<&str> = commits.iter().map(|s| s.as_str()).collect();
    cherry_pick_args.extend(commits_refs.iter().copied());

    let cherry_pick_output = git_cmd(&model.workdir, &cherry_pick_args).output();

    match cherry_pick_output {
        Err(e) => {
            model.popup = Some(PopupContent::Error {
                message: format!("{}: {}", i18n::t().err_run_cherry_pick_failed, e),
            });
            return Some(Message::Refresh);
        }
        Ok(output) if !output.status.success() => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            model.popup = Some(PopupContent::Error {
                message: format!(
                    "{}:\n{}",
                    i18n::t().err_cherry_pick_conflicts,
                    stderr.trim()
                ),
            });
            return Some(Message::Refresh);
        }
        Ok(_) => {}
    }

    // Step 3: switch back to the original branch
    let Some(ref src) = current_branch else {
        return Some(Message::Refresh);
    };

    let checkout_back = git_cmd(&model.workdir, &["checkout", src.as_str()]).output();

    match checkout_back {
        Err(e) => {
            model.popup = Some(PopupContent::Error {
                message: format!(
                    "{}: {}",
                    i18n::t().fmt1(i18n::t().err_cherry_pick_return_failed_fmt, src),
                    e
                ),
            });
            return Some(Message::Refresh);
        }
        Ok(output) if !output.status.success() => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            model.popup = Some(PopupContent::Error {
                message: format!(
                    "{}: {}",
                    i18n::t().fmt1(i18n::t().err_cherry_pick_return_failed_fmt, src),
                    stderr.trim()
                ),
            });
            return Some(Message::Refresh);
        }
        Ok(_) => {}
    }

    // Step 4: remove donated commits from the original branch via rebase --onto
    // `git rebase --onto <oldest>^ <latest> <src_branch>`
    let oldest = &commits[0];
    let latest = &commits[commits.len() - 1];
    let oldest_parent = format!("{}^", oldest);

    let rebase_output = git_cmd(
        &model.workdir,
        &[
            "rebase",
            "--onto",
            &oldest_parent,
            latest.as_str(),
            src.as_str(),
        ],
    )
    .output();

    match rebase_output {
        Err(e) => {
            model.popup = Some(PopupContent::Error {
                message: format!("{}: {}", i18n::t().err_donated_cleanup_failed, e),
            });
        }
        Ok(output) if !output.status.success() => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            model.popup = Some(PopupContent::Error {
                message: format!(
                    "{}:\n{}",
                    i18n::t().err_donated_cleanup_failed,
                    stderr.trim()
                ),
            });
        }
        Ok(_) => {}
    }

    Some(Message::Refresh)
}
