use std::path::Path;
use std::process::Stdio;

use crate::{
    git::{
        git_cmd,
        releases::{list_releases, parse_release_tag},
    },
    model::{Model, popup::PopupContent},
    msg::{Message, update::create_tag::take_tag_flags},
};

/// The repository directory name with each word capitalized, used in the
/// fallback annotation message (e.g. "my-repo" becomes "My-Repo").
fn capitalized_repo_name(workdir: &Path) -> String {
    let name = workdir
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();

    let mut capitalized = String::with_capacity(name.len());
    let mut at_word_start = true;
    for c in name.chars() {
        if c.is_alphanumeric() {
            if at_word_start {
                capitalized.extend(c.to_uppercase());
            } else {
                capitalized.push(c);
            }
            at_word_start = false;
        } else {
            capitalized.push(c);
            at_word_start = true;
        }
    }
    capitalized
}

/// Create a release tag named `name` at HEAD, mirroring magit's
/// `magit-tag-release`.
///
/// For annotated (or signed) tags the message is derived from the previous
/// release tag's message with the version string substituted, falling back
/// to "<Repo> <version>". The first release forces --edit so the user can
/// review the proposed message.
pub fn update(model: &mut Model, name: String) -> Option<Message> {
    let mut flags = take_tag_flags(model);

    let releases = list_releases(&model.workdir);
    let previous = releases.first();

    if previous.is_none() && !flags.iter().any(|flag| flag == "--edit") {
        // Keep the stable flag order: --force before --edit
        let at = usize::from(flags.first().is_some_and(|flag| flag == "--force"));
        flags.insert(at, "--edit".to_string());
    }

    let version = parse_release_tag(&name).map(|(_, version)| version);

    let annotated = flags
        .iter()
        .any(|flag| flag == "--annotate" || flag == "--sign");
    let message = annotated.then(|| match (previous, &version) {
        (Some(prev), Some(version)) if prev.message.contains(&prev.version) => {
            prev.message.replacen(&prev.version, version, 1)
        }
        (Some(prev), _) if prev.message.contains(&prev.tag) => {
            prev.message.replacen(&prev.tag, &name, 1)
        }
        _ => format!(
            "{} {}",
            capitalized_repo_name(&model.workdir),
            version.as_deref().unwrap_or(&name)
        ),
    });

    let mut args = vec!["tag".to_string()];
    args.extend(flags.iter().cloned());
    if let Some(message) = &message {
        args.extend(["-m".to_string(), message.clone()]);
    }
    args.push(name.clone());

    // Only --edit opens the editor: --annotate/--sign get their message via
    // -m, and the first release always carries --edit (added above)
    if flags.iter().any(|flag| flag == "--edit") {
        return Some(Message::CreateTagWithEditor { name, args });
    }

    let args: Vec<&str> = args.iter().map(String::as_str).collect();

    let output = git_cmd(&model.workdir, &args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output();

    model.popup = None;

    match output {
        Ok(out) if out.status.success() => Some(Message::Refresh),
        Ok(out) => {
            let stderr = String::from_utf8_lossy(&out.stderr).trim().to_string();
            model.popup = Some(PopupContent::Error {
                message: format!("Failed to create tag '{}': {}", name, stderr),
            });
            None
        }
        Err(err) => {
            model.popup = Some(PopupContent::Error {
                message: format!("Failed to create tag '{}': {}", name, err),
            });
            None
        }
    }
}
