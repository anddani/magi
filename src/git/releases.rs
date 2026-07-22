use std::path::Path;
use std::process::Stdio;

use lazy_static::lazy_static;
use regex::Regex;

use crate::git::git_cmd;

lazy_static! {
    /// Mirrors magit's `magit-release-tag-regexp`: an optional
    /// "v"/"version"/"r"/"release" prefix (with optional separator)
    /// followed by a dot-separated version, e.g. "v1.2.3" or "1.0.0-rc.1".
    /// Capture 1 is the prefix (including separator), capture 2 the version.
    static ref RELEASE_TAG_REGEX: Regex = Regex::new(
        r"(?i)^((?:v(?:ersion)?|r(?:elease)?)[-_/]?)?([0-9]+(?:\.[0-9]+)*(?:-[a-zA-Z0-9-]+(?:\.[a-zA-Z0-9-]+)*)?)$"
    )
    .unwrap();

    /// Mirrors magit's `magit-release-commit-regexp`: commit subjects of the
    /// form "Release version <version>". Capture 1 is the version.
    static ref RELEASE_COMMIT_REGEX: Regex =
        Regex::new(r"(?i)^Release version (.+)$").unwrap();
}

/// A release tag, as understood by the tag-release command.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Release {
    /// The version string extracted from the tag (e.g. "1.2.3" for "v1.2.3")
    pub version: String,
    /// The full tag name (e.g. "v1.2.3")
    pub tag: String,
    /// The tag's one-line message as printed by `git tag -n` (the annotation
    /// subject for annotated tags, the commit subject for lightweight tags)
    pub message: String,
}

/// Splits a release tag into its prefix (including the separator, e.g. "v")
/// and version string (e.g. "1.2.3"). Returns `None` for non-release tags.
pub fn parse_release_tag(tag: &str) -> Option<(String, String)> {
    RELEASE_TAG_REGEX.captures(tag).map(|caps| {
        (
            caps.get(1).map(|m| m.as_str()).unwrap_or("").to_string(),
            caps[2].to_string(),
        )
    })
}

/// Extracts the version from a "Release version <version>" commit subject.
pub fn parse_release_commit_subject(subject: &str) -> Option<String> {
    RELEASE_COMMIT_REGEX
        .captures(subject)
        .map(|caps| caps[1].to_string())
}

/// Numeric sort key for a version string, mirroring Emacs' `version-to-list`
/// with magit's `magit-tag-version-regexp-alist`: numeric components keep
/// their value, pre-release words map to negative values so that e.g.
/// "1.0-rc.1" sorts before "1.0".
fn version_key(version: &str) -> Vec<i64> {
    let mut key = Vec::new();
    let mut chars = version.chars().peekable();
    while let Some(&c) = chars.peek() {
        if c.is_ascii_digit() {
            let mut n: i64 = 0;
            while let Some(&d) = chars.peek() {
                if let Some(digit) = d.to_digit(10) {
                    n = n.saturating_mul(10).saturating_add(digit as i64);
                    chars.next();
                } else {
                    break;
                }
            }
            key.push(n);
        } else if c.is_alphabetic() {
            let mut word = String::new();
            while let Some(&a) = chars.peek() {
                if a.is_alphabetic() {
                    word.push(a.to_ascii_lowercase());
                    chars.next();
                } else {
                    break;
                }
            }
            key.push(match word.as_str() {
                "alpha" => -3,
                "beta" => -2,
                "pre" | "rc" => -1,
                // snapshot, cvs, git, unknown, and anything unrecognised
                _ => -4,
            });
        } else {
            // Separator ('.', '-', '_', '+', ' ')
            chars.next();
        }
    }
    key
}

/// Compares two version strings, padding the shorter key with zeros so that
/// "1.0" < "1.0.1" and "1.0-rc.1" < "1.0".
fn compare_versions(a: &str, b: &str) -> std::cmp::Ordering {
    let (ka, kb) = (version_key(a), version_key(b));
    let len = ka.len().max(kb.len());
    for i in 0..len {
        let (va, vb) = (
            ka.get(i).copied().unwrap_or(0),
            kb.get(i).copied().unwrap_or(0),
        );
        match va.cmp(&vb) {
            std::cmp::Ordering::Equal => continue,
            other => return other,
        }
    }
    std::cmp::Ordering::Equal
}

/// Returns all release tags, highest version first, mirroring magit's
/// `magit--list-releases`. Parses `git tag -n` because the tag message is
/// needed to propose an annotation message for the next release.
pub fn list_releases(workdir: &Path) -> Vec<Release> {
    let output = git_cmd(workdir, &["tag", "-n"])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output();

    let Ok(output) = output else {
        return vec![];
    };
    if !output.status.success() {
        return vec![];
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut releases: Vec<Release> = stdout
        .lines()
        .filter_map(|line| {
            let (tag, message) = match line.find(char::is_whitespace) {
                Some(idx) => (&line[..idx], line[idx..].trim_start()),
                None => (line, ""),
            };
            parse_release_tag(tag).map(|(_, version)| Release {
                version,
                tag: tag.to_string(),
                message: message.to_string(),
            })
        })
        .collect();

    releases.sort_by(|a, b| compare_versions(&b.version, &a.version));
    releases
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_release_tags_with_and_without_prefix() {
        assert_eq!(
            parse_release_tag("v1.2.3"),
            Some(("v".to_string(), "1.2.3".to_string()))
        );
        assert_eq!(
            parse_release_tag("1.2.3"),
            Some((String::new(), "1.2.3".to_string()))
        );
        assert_eq!(
            parse_release_tag("release/2.0"),
            Some(("release/".to_string(), "2.0".to_string()))
        );
        assert_eq!(
            parse_release_tag("v1.0.0-rc.1"),
            Some(("v".to_string(), "1.0.0-rc.1".to_string()))
        );
    }

    #[test]
    fn rejects_non_release_tags() {
        assert_eq!(parse_release_tag("nightly"), None);
        assert_eq!(parse_release_tag("v1.2.3+build"), None);
        assert_eq!(parse_release_tag("foo-1.0"), None);
    }

    #[test]
    fn parses_release_commit_subjects() {
        assert_eq!(
            parse_release_commit_subject("Release version 1.2.3"),
            Some("1.2.3".to_string())
        );
        assert_eq!(parse_release_commit_subject("Bump version to 1.2.3"), None);
    }

    #[test]
    fn orders_versions_numerically_not_lexically() {
        use std::cmp::Ordering::Less;
        assert_eq!(compare_versions("1.9.0", "1.10.0"), Less);
        assert_eq!(compare_versions("1.0", "1.0.1"), Less);
        assert_eq!(compare_versions("0.9", "1.0"), Less);
    }

    #[test]
    fn orders_prereleases_before_the_release() {
        use std::cmp::Ordering::Less;
        assert_eq!(compare_versions("1.0.0-alpha", "1.0.0-beta"), Less);
        assert_eq!(compare_versions("1.0.0-beta", "1.0.0-rc.1"), Less);
        assert_eq!(compare_versions("1.0.0-rc.1", "1.0.0"), Less);
    }
}
