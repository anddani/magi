use crate::{
    git::{
        read_commit_message,
        releases::{list_releases, parse_release_commit_subject, parse_release_tag},
    },
    model::{
        Model,
        popup::{InputContext, InputPopupState, PopupContent},
    },
    msg::Message,
};

/// Proposes the next release tag for HEAD, mirroring magit's
/// `magit-tag-release`:
///
/// - If HEAD's commit message matches "Release version <v>" and a release
///   tag already exists, the tag is created directly as `<prefix><v>`,
///   reusing the previous tag's prefix (e.g. "v").
/// - Otherwise an input popup is shown, prefilled with the previous release
///   tag (leaving it to the user to increment the desired part), or with a
///   name derived from HEAD's message for the very first release.
pub fn update(model: &mut Model) -> Option<Message> {
    let releases = list_releases(&model.workdir);
    let subject = read_commit_message(&model.workdir, "HEAD").unwrap_or_default();
    let version = parse_release_commit_subject(&subject);

    match (releases.first(), version) {
        (Some(previous), Some(version)) => {
            let prefix = parse_release_tag(&previous.tag)
                .map(|(prefix, _)| prefix)
                .unwrap_or_default();
            Some(Message::CreateTagRelease {
                name: format!("{}{}", prefix, version),
            })
        }
        (Some(previous), None) => {
            model.popup = Some(PopupContent::Input(InputPopupState::with_text(
                InputContext::TagRelease {
                    previous: Some(previous.tag.clone()),
                },
                previous.tag.clone(),
            )));
            None
        }
        (None, version) => {
            // First release: prefix plain "1.0"-style versions with "v"
            let initial = match version {
                Some(v) if v.starts_with(|c: char| c.is_ascii_digit()) => format!("v{}", v),
                Some(v) => v,
                None => String::new(),
            };
            model.popup = Some(PopupContent::Input(InputPopupState::with_text(
                InputContext::TagRelease { previous: None },
                initial,
            )));
            None
        }
    }
}
