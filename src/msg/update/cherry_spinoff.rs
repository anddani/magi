use crate::{
    git::git_cmd,
    i18n,
    model::{Model, popup::PopupContent},
    msg::Message,
};

pub fn update(
    model: &mut Model,
    commits: Vec<String>,
    branch: String,
    root: String,
) -> Option<Message> {
    model.popup = None;

    if commits.is_empty() {
        return None;
    }

    // Remember the current branch so we can return to it for cleanup
    let current_branch: Option<String> =
        git_cmd(&model.workdir, &["rev-parse", "--abbrev-ref", "HEAD"])
            .output()
            .ok()
            .filter(|o| o.status.success())
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
            .filter(|s| s != "HEAD"); // ignore detached HEAD

    // Step 1: create the new branch at the user-chosen root
    let oldest = &commits[0];
    let start_point = root;

    let create_output = git_cmd(
        &model.workdir,
        &["branch", branch.as_str(), start_point.as_str()],
    )
    .output();

    match create_output {
        Err(e) => {
            model.popup = Some(PopupContent::Error {
                message: format!(
                    "{}: {}",
                    i18n::t().fmt1(i18n::t().err_create_branch_named_failed_fmt, &branch),
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
                    i18n::t().fmt1(i18n::t().err_create_branch_named_failed_fmt, &branch),
                    stderr.trim()
                ),
            });
            return Some(Message::Refresh);
        }
        Ok(_) => {}
    }

    // Step 2: checkout the new branch
    let checkout_output = git_cmd(&model.workdir, &["checkout", branch.as_str()]).output();

    match checkout_output {
        Err(e) => {
            model.popup = Some(PopupContent::Error {
                message: format!(
                    "{}: {}",
                    i18n::t().fmt1(i18n::t().err_checkout_named_failed_fmt, &branch),
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
                    i18n::t().fmt1(i18n::t().err_checkout_named_failed_fmt, &branch),
                    stderr.trim()
                ),
            });
            return Some(Message::Refresh);
        }
        Ok(_) => {}
    }

    // Step 3: cherry-pick the commits onto the new branch
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

    // Step 4: switch back to the original branch for cleanup
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

    // Step 5: remove the spun-off commits from the original branch via rebase --onto
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
                message: format!("{}: {}", i18n::t().err_spun_off_cleanup_failed, e),
            });
            return Some(Message::Refresh);
        }
        Ok(output) if !output.status.success() => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            model.popup = Some(PopupContent::Error {
                message: format!(
                    "{}:\n{}",
                    i18n::t().err_spun_off_cleanup_failed,
                    stderr.trim()
                ),
            });
            return Some(Message::Refresh);
        }
        Ok(_) => {}
    }

    // Step 6: checkout the new branch (spinoff ends up on the new branch)
    let _ = git_cmd(&model.workdir, &["checkout", branch.as_str()]).output();

    Some(Message::Refresh)
}
