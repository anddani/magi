// Define the module structure
pub mod get_head_ref;
pub mod get_latest_tag;
pub mod get_merge_ref;
pub mod get_push_ref;

// Re-export the functions from the submodules
pub use get_head_ref::get_head_ref;
pub use get_latest_tag::get_latest_tag;
pub use get_merge_ref::get_merge_ref;
pub use get_push_ref::get_push_ref;

use crate::{
    errors::MagiResult,
    model::{Line, LineContent, SectionType},
};
use git2::Repository;

use super::TagInfo;

/// Returns the lines representing the current state of the Git repository
pub fn get_lines(repository: &Repository) -> MagiResult<Vec<Line>> {
    let mut lines = Vec::new();

    // Get the head reference
    let head_ref = get_head_ref(repository)?;
    lines.push(Line {
        content: LineContent::HeadRef(head_ref),
        section: Some(SectionType::Info),
    });

    // Get the merge (upstream) reference
    if let Some(merge_ref) = get_merge_ref(repository)? {
        lines.push(Line {
            content: LineContent::MergeRef(merge_ref),
            section: Some(SectionType::Info),
        });
    }

    // Get the push remote reference (only when pushRemote/pushDefault is configured)
    if let Some(push_ref) = get_push_ref(repository)? {
        lines.push(Line {
            content: LineContent::PushRef(push_ref),
            section: Some(SectionType::Info),
        });
    }

    // Get the latest tag
    if let Some(tag_info) = get_latest_tag(repository)? {
        lines.push(Line {
            content: LineContent::Tag(TagInfo {
                name: tag_info.name,
                commits_ahead: tag_info.commits_ahead,
            }),
            section: Some(SectionType::Info),
        });
    }

    Ok(lines)
}
