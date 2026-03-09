use std::path::Path;

use crate::{
    errors::MagiResult,
    git::{
        commit::{CommitResult, get_commit_result},
        git_cmd,
    },
};

pub fn run_merge_continue_with_editor<P: AsRef<Path>>(repo_path: P) -> MagiResult<CommitResult> {
    let status = git_cmd(&repo_path, &["merge", "--continue"]).status()?;

    get_commit_result(repo_path, status, "Merge continue")
}
