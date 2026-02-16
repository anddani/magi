use std::path::Path;
use std::process::{Command, Stdio};

use regex::Regex;

use super::git_cmd;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HostingService {
    GitHub,
    GitLab,
    Bitbucket,
    AzureDevOps,
    Gitea,
    Codeberg,
}

/// Runs `git ls-remote --get-url origin` to get the remote URL.
pub fn get_remote_url<P: AsRef<Path>>(repo_path: P) -> Result<String, String> {
    let output = git_cmd(&repo_path, &["ls-remote", "--get-url", "origin"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .map_err(|e| format!("Failed to run git: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(if stderr.is_empty() {
            "Failed to get remote URL".to_string()
        } else {
            stderr
        });
    }

    let url = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if url.is_empty() {
        return Err("No remote URL found for origin".to_string());
    }
    Ok(url)
}

/// Parses a remote URL (SSH or HTTPS) and extracts (host, owner, repo).
pub fn parse_remote_url(url: &str) -> Result<(String, String, String), String> {
    // SSH: git@host:owner/repo.git
    let ssh_re = Regex::new(r"^[^@]+@([^:]+):(.+)/([^/]+?)(?:\.git)?$").unwrap();
    if let Some(caps) = ssh_re.captures(url) {
        let host = caps[1].to_string();
        let owner = caps[2].to_string();
        let repo = caps[3].to_string();
        return Ok((host, owner, repo));
    }

    // HTTPS: https://host/owner/repo.git or https://host/owner/repo
    // Also handle Azure DevOps: https://dev.azure.com/org/project/_git/repo
    let https_re = Regex::new(r"^https?://([^/]+)/(.+)/([^/]+?)(?:\.git)?$").unwrap();
    if let Some(caps) = https_re.captures(url) {
        let host = caps[1].to_string();
        let path = caps[2].to_string();
        let repo = caps[3].to_string();

        // Azure DevOps has a different path structure: org/project/_git/repo
        // The regex captures org/project/_git as the "owner" part
        if host.contains("dev.azure.com")
            && let Some(git_idx) = path.find("/_git")
        {
            let owner = path[..git_idx].to_string();
            return Ok((host, owner, repo));
        }

        return Ok((host, path, repo));
    }

    Err(format!("Could not parse remote URL: {}", url))
}

/// Detects the hosting service from the host string.
pub fn detect_service(host: &str) -> Option<HostingService> {
    let host_lower = host.to_lowercase();
    if host_lower.contains("github") {
        Some(HostingService::GitHub)
    } else if host_lower.contains("gitlab") {
        Some(HostingService::GitLab)
    } else if host_lower.contains("bitbucket") {
        Some(HostingService::Bitbucket)
    } else if host_lower.contains("dev.azure.com") || host_lower.contains("visualstudio.com") {
        Some(HostingService::AzureDevOps)
    } else if host_lower.contains("codeberg") {
        Some(HostingService::Codeberg)
    } else if host_lower.contains("gitea") {
        Some(HostingService::Gitea)
    } else {
        None
    }
}

/// Builds the URL for creating a pull request on the given hosting service.
///
/// `from` is the source branch name (URL-encoded).
/// `to` is the optional target branch name. If `None`, the service default is used.
pub fn build_pr_url(
    service: &HostingService,
    host: &str,
    owner: &str,
    repo: &str,
    from: &str,
    to: Option<&str>,
) -> String {
    let from_encoded = urlencoding::encode(from);

    match service {
        HostingService::GitHub => {
            // https://github.com/owner/repo/compare/main...feature
            match to {
                Some(target) => {
                    let to_encoded = urlencoding::encode(target);
                    format!(
                        "https://{}/{}/{}/compare/{}...{}",
                        host, owner, repo, to_encoded, from_encoded
                    )
                }
                None => {
                    format!(
                        "https://{}/{}/{}/compare/{}?expand=1",
                        host, owner, repo, from_encoded
                    )
                }
            }
        }
        HostingService::GitLab => {
            // https://gitlab.com/owner/repo/-/merge_requests/new?merge_request[source_branch]=feature
            let mut url = format!(
                "https://{}/{}/{}/-/merge_requests/new?merge_request[source_branch]={}",
                host, owner, repo, from_encoded
            );
            if let Some(target) = to {
                let to_encoded = urlencoding::encode(target);
                url.push_str(&format!("&merge_request[target_branch]={}", to_encoded));
            }
            url
        }
        HostingService::Bitbucket => {
            // https://bitbucket.org/owner/repo/pull-requests/new?source=feature&dest=main
            let mut url = format!(
                "https://{}/{}/{}/pull-requests/new?source={}",
                host, owner, repo, from_encoded
            );
            if let Some(target) = to {
                let to_encoded = urlencoding::encode(target);
                url.push_str(&format!("&dest={}", to_encoded));
            }
            url
        }
        HostingService::AzureDevOps => {
            // https://dev.azure.com/org/project/_git/repo/pullrequestcreate?sourceRef=feature&targetRef=main
            let mut url = format!(
                "https://{}/{}/_git/{}/pullrequestcreate?sourceRef={}",
                host, owner, repo, from_encoded
            );
            if let Some(target) = to {
                let to_encoded = urlencoding::encode(target);
                url.push_str(&format!("&targetRef={}", to_encoded));
            }
            url
        }
        HostingService::Gitea | HostingService::Codeberg => {
            // https://codeberg.org/owner/repo/compare/main...feature
            match to {
                Some(target) => {
                    let to_encoded = urlencoding::encode(target);
                    format!(
                        "https://{}/{}/{}/compare/{}...{}",
                        host, owner, repo, to_encoded, from_encoded
                    )
                }
                None => {
                    format!(
                        "https://{}/{}/{}/compare/{}",
                        host, owner, repo, from_encoded
                    )
                }
            }
        }
    }
}

/// Opens a URL in the system's default browser.
pub fn open_in_browser(url: &str) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    let result = Command::new("open")
        .arg(url)
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .output();

    #[cfg(target_os = "linux")]
    let result = Command::new("xdg-open")
        .arg(url)
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .output();

    #[cfg(target_os = "windows")]
    let result = Command::new("cmd")
        .args(["/C", "start", url])
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .output();

    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    let result: Result<std::process::Output, std::io::Error> = Err(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        "Unsupported platform",
    ));

    match result {
        Ok(output) if output.status.success() => Ok(()),
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            Err(format!("Failed to open browser: {}", stderr))
        }
        Err(e) => Err(format!("Failed to open browser: {}", e)),
    }
}

/// Checks if the given branch has an upstream (remote tracking branch) configured.
pub fn has_upstream<P: AsRef<Path>>(repo_path: P, branch: &str) -> bool {
    let key = format!("branch.{}.remote", branch);
    git_cmd(&repo_path, &["config", "--get", &key])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .output()
        .is_ok_and(|output| output.status.success())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_ssh_url() {
        let (host, owner, repo) = parse_remote_url("git@github.com:user/my-repo.git").unwrap();
        assert_eq!(host, "github.com");
        assert_eq!(owner, "user");
        assert_eq!(repo, "my-repo");
    }

    #[test]
    fn test_parse_ssh_url_without_git_suffix() {
        let (host, owner, repo) = parse_remote_url("git@github.com:user/my-repo").unwrap();
        assert_eq!(host, "github.com");
        assert_eq!(owner, "user");
        assert_eq!(repo, "my-repo");
    }

    #[test]
    fn test_parse_https_url() {
        let (host, owner, repo) = parse_remote_url("https://github.com/user/my-repo.git").unwrap();
        assert_eq!(host, "github.com");
        assert_eq!(owner, "user");
        assert_eq!(repo, "my-repo");
    }

    #[test]
    fn test_parse_https_url_without_git_suffix() {
        let (host, owner, repo) = parse_remote_url("https://github.com/user/my-repo").unwrap();
        assert_eq!(host, "github.com");
        assert_eq!(owner, "user");
        assert_eq!(repo, "my-repo");
    }

    #[test]
    fn test_parse_gitlab_subgroup_url() {
        let (host, owner, repo) =
            parse_remote_url("git@gitlab.com:group/subgroup/my-repo.git").unwrap();
        assert_eq!(host, "gitlab.com");
        assert_eq!(owner, "group/subgroup");
        assert_eq!(repo, "my-repo");
    }

    #[test]
    fn test_parse_azure_devops_url() {
        let (host, owner, repo) =
            parse_remote_url("https://dev.azure.com/org/project/_git/my-repo").unwrap();
        assert_eq!(host, "dev.azure.com");
        assert_eq!(owner, "org/project");
        assert_eq!(repo, "my-repo");
    }

    #[test]
    fn test_parse_invalid_url() {
        assert!(parse_remote_url("not-a-url").is_err());
    }

    #[test]
    fn test_detect_github() {
        assert_eq!(detect_service("github.com"), Some(HostingService::GitHub));
    }

    #[test]
    fn test_detect_gitlab() {
        assert_eq!(detect_service("gitlab.com"), Some(HostingService::GitLab));
        assert_eq!(
            detect_service("gitlab.mycompany.com"),
            Some(HostingService::GitLab)
        );
    }

    #[test]
    fn test_detect_bitbucket() {
        assert_eq!(
            detect_service("bitbucket.org"),
            Some(HostingService::Bitbucket)
        );
    }

    #[test]
    fn test_detect_azure() {
        assert_eq!(
            detect_service("dev.azure.com"),
            Some(HostingService::AzureDevOps)
        );
    }

    #[test]
    fn test_detect_codeberg() {
        assert_eq!(
            detect_service("codeberg.org"),
            Some(HostingService::Codeberg)
        );
    }

    #[test]
    fn test_detect_gitea() {
        assert_eq!(
            detect_service("gitea.example.com"),
            Some(HostingService::Gitea)
        );
    }

    #[test]
    fn test_detect_unknown() {
        assert_eq!(detect_service("example.com"), None);
    }

    #[test]
    fn test_build_github_pr_url_no_target() {
        let url = build_pr_url(
            &HostingService::GitHub,
            "github.com",
            "user",
            "repo",
            "feature/my-branch",
            None,
        );
        assert_eq!(
            url,
            "https://github.com/user/repo/compare/feature%2Fmy-branch?expand=1"
        );
    }

    #[test]
    fn test_build_github_pr_url_with_target() {
        let url = build_pr_url(
            &HostingService::GitHub,
            "github.com",
            "user",
            "repo",
            "feature",
            Some("main"),
        );
        assert_eq!(url, "https://github.com/user/repo/compare/main...feature");
    }

    #[test]
    fn test_build_gitlab_pr_url_no_target() {
        let url = build_pr_url(
            &HostingService::GitLab,
            "gitlab.com",
            "user",
            "repo",
            "feature",
            None,
        );
        assert_eq!(
            url,
            "https://gitlab.com/user/repo/-/merge_requests/new?merge_request[source_branch]=feature"
        );
    }

    #[test]
    fn test_build_gitlab_pr_url_with_target() {
        let url = build_pr_url(
            &HostingService::GitLab,
            "gitlab.com",
            "user",
            "repo",
            "feature",
            Some("develop"),
        );
        assert_eq!(
            url,
            "https://gitlab.com/user/repo/-/merge_requests/new?merge_request[source_branch]=feature&merge_request[target_branch]=develop"
        );
    }

    #[test]
    fn test_build_bitbucket_pr_url_with_target() {
        let url = build_pr_url(
            &HostingService::Bitbucket,
            "bitbucket.org",
            "user",
            "repo",
            "feature",
            Some("main"),
        );
        assert_eq!(
            url,
            "https://bitbucket.org/user/repo/pull-requests/new?source=feature&dest=main"
        );
    }

    #[test]
    fn test_build_azure_pr_url_with_target() {
        let url = build_pr_url(
            &HostingService::AzureDevOps,
            "dev.azure.com",
            "org/project",
            "repo",
            "feature",
            Some("main"),
        );
        assert_eq!(
            url,
            "https://dev.azure.com/org/project/_git/repo/pullrequestcreate?sourceRef=feature&targetRef=main"
        );
    }

    #[test]
    fn test_build_codeberg_pr_url_with_target() {
        let url = build_pr_url(
            &HostingService::Codeberg,
            "codeberg.org",
            "user",
            "repo",
            "feature",
            Some("main"),
        );
        assert_eq!(url, "https://codeberg.org/user/repo/compare/main...feature");
    }

    #[test]
    fn test_build_codeberg_pr_url_no_target() {
        let url = build_pr_url(
            &HostingService::Codeberg,
            "codeberg.org",
            "user",
            "repo",
            "feature",
            None,
        );
        assert_eq!(url, "https://codeberg.org/user/repo/compare/feature");
    }
}
