use crate::{
    model::{Model, arguments::Arguments::PushArguments},
    msg::Message,
};

/// Parse remote/branch into components.
/// e.g., "origin/main" -> ("origin", "main")
fn parse_remote_branch(upstream: &str) -> (String, String) {
    if let Some((remote, branch)) = upstream.split_once('/') {
        (remote.to_string(), branch.to_string())
    } else {
        // If no slash, assume it's just the remote name
        (upstream.to_string(), String::new())
    }
}

pub fn update(model: &mut Model, upstream: String) -> Option<Message> {
    let (remote, branch) = parse_remote_branch(&upstream);

    // Build the refspec for setting upstream
    let refspec = format!("HEAD:{}", branch);

    // Build extra arguments for setting upstream
    let mut args = vec![
        "push".to_string(),
        "-v".to_string(),
        "--set-upstream".to_string(),
        remote,
        refspec,
    ];

    if let Some(PushArguments(arguments)) = model.arguments.take() {
        args.extend(arguments.into_iter().map(|a| a.flag().to_string()));
    }

    let operation_name = format!("Push to {}", upstream);

    super::pty_helper::execute_pty_command(model, args, operation_name)
}
