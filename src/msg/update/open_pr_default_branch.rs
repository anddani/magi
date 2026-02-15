use crate::{
    git::open_pr::{
        build_pr_url, detect_service, get_remote_url, has_upstream, open_in_browser,
        parse_remote_url,
    },
    model::{Model, popup::PopupContent},
    msg::Message,
};

pub fn update(model: &mut Model) -> Option<Message> {
    model.popup = None;

    let repo_path = match model.git_info.repository.workdir() {
        Some(path) => path.to_path_buf(),
        None => {
            model.popup = Some(PopupContent::Error {
                message: "Could not determine repository path".to_string(),
            });
            return None;
        }
    };

    let current_branch = match model.git_info.current_branch() {
        Some(branch) => branch,
        None => {
            model.popup = Some(PopupContent::Error {
                message: "Could not determine current branch (detached HEAD?)".to_string(),
            });
            return None;
        }
    };

    if !has_upstream(&repo_path, &current_branch) {
        model.popup = Some(PopupContent::Error {
            message: format!(
                "Branch '{}' has no upstream. Push it first.",
                current_branch
            ),
        });
        return None;
    }

    let remote_url = match get_remote_url(&repo_path) {
        Ok(url) => url,
        Err(e) => {
            model.popup = Some(PopupContent::Error { message: e });
            return None;
        }
    };

    let (host, owner, repo) = match parse_remote_url(&remote_url) {
        Ok(parsed) => parsed,
        Err(e) => {
            model.popup = Some(PopupContent::Error { message: e });
            return None;
        }
    };

    let service = match detect_service(&host) {
        Some(s) => s,
        None => {
            model.popup = Some(PopupContent::Error {
                message: format!("Unsupported hosting service: {}", host),
            });
            return None;
        }
    };

    let url = build_pr_url(&service, &host, &owner, &repo, &current_branch, None);

    if let Err(e) = open_in_browser(&url) {
        model.popup = Some(PopupContent::Error { message: e });
    }

    None
}
