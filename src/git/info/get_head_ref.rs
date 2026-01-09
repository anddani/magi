use crate::git::{GitRef, ReferenceType};
use git2::Repository;

/// Get the HEAD reference information from a Git repository
pub fn get_head_ref(repo: &Repository) -> Result<GitRef, git2::Error> {
    let head = repo.head()?;

    // If HEAD is a branch, use the branch name and commit info
    if head.is_branch() {
        let name = head.shorthand().unwrap_or("unknown").to_string();
        let commit = head.peel_to_commit()?;
        let commit_hash = commit.id().to_string();
        let short_hash = commit_hash.chars().take(7).collect::<String>();
        let commit_message = commit.message().unwrap_or("").to_string();

        // Determine if it's a local or remote branch
        let reference_type = if name.starts_with("origin/") {
            ReferenceType::RemoteBranch
        } else {
            ReferenceType::LocalBranch
        };

        Ok(GitRef::new(
            name,
            short_hash,
            commit_message,
            reference_type,
        ))
    } else {
        // Detached HEAD
        let commit = head.peel_to_commit()?;
        let commit_hash = commit.id().to_string();
        let short_hash = commit_hash.chars().take(7).collect::<String>();
        let commit_message = commit.message().unwrap_or("").to_string();

        Ok(GitRef::new_detached_head(short_hash, commit_message))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::git::test_repo;

    #[test]
    fn test_get_head_ref_attached() -> Result<(), git2::Error> {
        // Test branch scenario
        let test_repo = test_repo::TestRepo::new();
        let repo = &test_repo.repo;
        let head_ref = get_head_ref(&repo)?;
        assert_eq!(head_ref.name, "main");
        assert_eq!(head_ref.reference_type, ReferenceType::LocalBranch);
        assert_eq!(head_ref.commit_hash.len(), 7);
        assert_eq!(head_ref.commit_message, "Initial commit");

        Ok(())
    }

    #[test]
    fn test_get_head_ref_detached() -> Result<(), git2::Error> {
        let test_repo = test_repo::TestRepo::new();
        test_repo.detach_head();
        let repo = test_repo.repo;
        let detached_head_ref = get_head_ref(&repo)?;
        assert_eq!(detached_head_ref.name, "HEAD (detached)");
        assert_eq!(
            detached_head_ref.reference_type,
            ReferenceType::DetachedHead
        );
        assert_eq!(detached_head_ref.commit_hash.len(), 7);
        assert_eq!(detached_head_ref.commit_message, "Initial commit");

        Ok(())
    }

    #[test]
    fn test_get_head_ref_remote_branch() -> Result<(), git2::Error> {
        let test_repo = test_repo::TestRepo::new();
        test_repo.create_remote_branch("main");
        let repo = &test_repo.repo;
        let head_ref = get_head_ref(&repo)?;
        assert_eq!(head_ref.name, "origin/main");
        assert_eq!(head_ref.reference_type, ReferenceType::RemoteBranch);
        assert_eq!(head_ref.commit_hash.len(), 7);
        assert_eq!(head_ref.commit_message, "Initial commit");

        Ok(())
    }
}
