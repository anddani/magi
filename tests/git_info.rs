use magi::git::info::{get_head_ref, get_push_ref};
use magi::git::test_repo::TestRepo;
use magi::git::ReferenceType;

#[test]
fn test_get_head_ref_attached() -> Result<(), git2::Error> {
    // Test branch scenario
    let test_repo = TestRepo::new();
    let repo = &test_repo.repo;
    let head_ref = get_head_ref(repo)?;
    assert_eq!(head_ref.name, "main");
    assert_eq!(head_ref.reference_type, ReferenceType::LocalBranch);
    assert_eq!(head_ref.commit_hash.len(), 7);
    assert_eq!(head_ref.commit_message, "Initial commit");

    Ok(())
}

#[test]
fn test_get_head_ref_detached() -> Result<(), git2::Error> {
    let test_repo = TestRepo::new();
    test_repo.detach_head();
    let repo = &test_repo.repo;
    let detached_head_ref = get_head_ref(repo)?;
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
    let test_repo = TestRepo::new();
    test_repo.create_remote_branch("main");
    let repo = &test_repo.repo;
    let head_ref = get_head_ref(repo)?;
    assert_eq!(head_ref.name, "origin/main");
    assert_eq!(head_ref.reference_type, ReferenceType::RemoteBranch);
    assert_eq!(head_ref.commit_hash.len(), 7);
    assert_eq!(head_ref.commit_message, "Initial commit");

    Ok(())
}

#[test]
fn test_get_push_ref_no_upstream() -> Result<(), git2::Error> {
    // Test case where there's no upstream branch configured
    let test_repo = TestRepo::new();
    let repo = &test_repo.repo;
    let push_ref = get_push_ref(repo)?;
    assert!(push_ref.is_none());

    Ok(())
}

#[test]
fn test_get_push_ref_detached_head() -> Result<(), git2::Error> {
    // Test case where HEAD is detached (should return None)
    let test_repo = TestRepo::new();
    test_repo.detach_head();
    let repo = &test_repo.repo;
    let push_ref = get_push_ref(repo)?;
    assert!(push_ref.is_none());

    Ok(())
}
