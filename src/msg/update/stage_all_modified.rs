use crate::{
    model::{DialogContent, LineContent, Model},
    msg::Message,
};

pub fn update(model: &mut Model) -> Option<Message> {
    let repo_path = model.git_info.repository.workdir()?;
    let files: Vec<&str> = model
        .ui_model
        .lines
        .iter()
        .filter_map(|line| match &line.content {
            LineContent::UnstagedFile(fc) => Some(fc.path.as_str()),
            _ => None,
        })
        .collect();
    if let Err(e) = crate::git::stage::stage_files(repo_path, &files) {
        model.dialog = Some(DialogContent::Error {
            message: format!("Error staging files: {}", e),
        });
    }
    Some(Message::Refresh)
}
