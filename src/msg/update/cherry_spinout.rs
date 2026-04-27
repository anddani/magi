use crate::{
    git::git_cmd,
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

    // Remember the current branch so we can return after spinning out
    let current_branch: Option<String> =
        git_cmd(&model.workdir, &["rev-parse", "--abbrev-ref", "HEAD"])
            .output()
            .ok()
            .filter(|o| o.status.success())
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
            .filter(|s| s != "HEAD"); // ignore detached HEAD

    // Step 1: create the new branch at the user-chosen root.
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
                message: format!("Failed to create branch '{}': {}", branch, e),
            });
            return Some(Message::Refresh);
        }
        Ok(output) if !output.status.success() => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            model.popup = Some(PopupContent::Error {
                message: format!("Failed to create branch '{}': {}", branch, stderr.trim()),
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
                message: format!("Failed to checkout '{}': {}", branch, e),
            });
            return Some(Message::Refresh);
        }
        Ok(output) if !output.status.success() => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            model.popup = Some(PopupContent::Error {
                message: format!("Failed to checkout '{}': {}", branch, stderr.trim()),
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
                message: format!("Failed to run cherry-pick: {}", e),
            });
            return Some(Message::Refresh);
        }
        Ok(output) if !output.status.success() => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            model.popup = Some(PopupContent::Error {
                message: format!(
                    "Cherry-pick failed (resolve conflicts manually):\n{}",
                    stderr.trim()
                ),
            });
            return Some(Message::Refresh);
        }
        Ok(_) => {}
    }

    // Step 4: switch back to the original branch
    let Some(ref src) = current_branch else {
        return Some(Message::Refresh);
    };

    let checkout_back = git_cmd(&model.workdir, &["checkout", src.as_str()]).output();

    match checkout_back {
        Err(e) => {
            model.popup = Some(PopupContent::Error {
                message: format!(
                    "Cherry-pick succeeded but failed to return to '{}': {}",
                    src, e
                ),
            });
            return Some(Message::Refresh);
        }
        Ok(output) if !output.status.success() => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            model.popup = Some(PopupContent::Error {
                message: format!(
                    "Cherry-pick succeeded but failed to return to '{}': {}",
                    src,
                    stderr.trim()
                ),
            });
            return Some(Message::Refresh);
        }
        Ok(_) => {}
    }

    // Step 5: remove the spun-out commits from the original branch via rebase --onto
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
                message: format!("Spun out but source cleanup failed: {}", e),
            });
        }
        Ok(output) if !output.status.success() => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            model.popup = Some(PopupContent::Error {
                message: format!("Spun out but source cleanup failed:\n{}", stderr.trim()),
            });
        }
        Ok(_) => {}
    }

    Some(Message::Refresh)
}
