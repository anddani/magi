use git2::{Repository, Signature};
use std::fs;
use tempfile::TempDir;

pub struct TestRepo {
    pub repo: Repository,
    _temp_dir: TempDir,
}

impl Default for TestRepo {
    fn default() -> Self {
        Self::new()
    }
}

impl TestRepo {
    pub fn new() -> Self {
        let local_dir = tempfile::tempdir().unwrap();
        let local_repo_path = local_dir.path();

        // Initialize a new Git repository
        let local_repo = Repository::init(local_repo_path).unwrap();

        // Set the default branch to main (libgit2 defaults to master)
        local_repo.set_head("refs/heads/main").unwrap();

        // Create a test file and commit it
        let file_path = local_repo_path.join("test.txt");
        fs::write(&file_path, "test content").unwrap();

        let mut index = local_repo.index().unwrap();
        index.add_path(std::path::Path::new("test.txt")).unwrap();
        index.write().unwrap();
        let tree_id = index.write_tree().unwrap();

        let signature = Signature::now("Test User", "test@example.com").unwrap();
        local_repo
            .commit(
                Some("refs/heads/main"),
                &signature,
                &signature,
                "Initial commit",
                &local_repo.find_tree(tree_id).unwrap(),
                &[],
            )
            .expect("Failed to commit to local repo");

        Self {
            repo: local_repo,
            _temp_dir: local_dir,
        }
    }

    pub fn detach_head(&self) {
        let commit = self.repo.head().unwrap().peel_to_commit().unwrap().id();
        self.repo.set_head_detached(commit).unwrap();
    }

    pub fn create_remote_branch(&self, branch_name: &str) {
        let commit = self.repo.head().unwrap().peel_to_commit().unwrap();

        // Create a remote reference
        let remote_ref_name = format!("refs/remotes/origin/{}", branch_name);
        self.repo
            .reference(&remote_ref_name, commit.id(), false, "Create remote branch")
            .unwrap();

        // Create a local branch with the remote name prefix to simulate tracking a remote
        let local_branch_name = format!("origin/{}", branch_name);
        let local_ref_name = format!("refs/heads/{}", local_branch_name);
        self.repo
            .reference(
                &local_ref_name,
                commit.id(),
                false,
                "Create local branch with remote name",
            )
            .unwrap();

        // Set HEAD to point to this branch
        self.repo.set_head(&local_ref_name).unwrap();
    }
}
