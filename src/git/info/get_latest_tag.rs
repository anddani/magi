use crate::git::TagInfo;
use git2::Error as Git2Error;

pub fn get_latest_tag(repository: &git2::Repository) -> Result<Option<TagInfo>, Git2Error> {
    let head = repository.head()?;
    let head_commit = head.peel_to_commit()?;

    let mut tags = Vec::new();
    repository.tag_foreach(|id, _| {
        if let Ok(tag) = repository.find_tag(id) {
            if let Ok(tag_commit) = tag.peel() {
                tags.push((tag.name().unwrap_or("").to_string(), tag_commit));
            }
        }
        true
    })?;

    // Find the most recent tag that points to the current HEAD or an ancestor
    let mut latest_tag = None;
    let mut latest_tag_commit = None;

    for (tag_name, commit) in tags {
        if commit.id() == head_commit.id() {
            latest_tag = Some(tag_name);
            latest_tag_commit = Some(commit);
            break;
        }
    }

    if let (Some(tag_name), Some(tag_commit)) = (latest_tag, latest_tag_commit) {
        let commits_ahead = repository
            .graph_ahead_behind(head_commit.id(), tag_commit.id())?
            .0;

        Ok(Some(TagInfo {
            name: tag_name,
            commits_ahead,
        }))
    } else {
        Ok(None)
    }
}
