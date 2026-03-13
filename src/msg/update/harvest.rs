use crate::{
    git::git_cmd,
    model::{Model, popup::PopupContent},
    msg::Message,
};

pub fn update(model: &mut Model, commits: Vec<String>, source: String) -> Option<Message> {
    model.popup = None;

    if commits.is_empty() {
        return None;
    }

    // Remember the current branch so we can switch back after the source cleanup
    let current_branch: Option<String> =
        git_cmd(&model.workdir, &["rev-parse", "--abbrev-ref", "HEAD"])
            .output()
            .ok()
            .filter(|o| o.status.success())
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
            .filter(|s| s != "HEAD"); // ignore detached HEAD

    // Step 1: cherry-pick the commits onto the current branch
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

    // Step 2: remove the harvested commits from the source branch via rebase --onto
    // `git rebase --onto <oldest>^ <latest> <source_branch>`
    // This removes the range oldest..latest (inclusive) from source_branch.
    // NOTE: git rebase with a branch argument will switch to that branch.
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
            source.as_str(),
        ],
    )
    .output();

    let rebase_ok = match rebase_output {
        Err(e) => {
            model.popup = Some(PopupContent::Error {
                message: format!("Cherry-pick succeeded but source cleanup failed: {}", e),
            });
            false
        }
        Ok(output) if !output.status.success() => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            model.popup = Some(PopupContent::Error {
                message: format!(
                    "Cherry-pick succeeded but source cleanup failed:\n{}",
                    stderr.trim()
                ),
            });
            false
        }
        Ok(_) => true,
    };

    // Step 3: switch back to the original branch (rebase --onto may have switched branches)
    if rebase_ok && let Some(branch) = &current_branch {
        let _ = git_cmd(&model.workdir, &["checkout", branch]).output();
    }

    Some(Message::Refresh)
}
