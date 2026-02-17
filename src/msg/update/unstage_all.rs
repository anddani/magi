use crate::{
    model::{LineContent, Model, popup::PopupContent},
    msg::Message,
};

pub fn update(model: &mut Model) -> Option<Message> {
    let repo_path = &model.workdir;
    let files: Vec<&str> = model
        .ui_model
        .lines
        .iter()
        .filter_map(|line| match &line.content {
            LineContent::StagedFile(fc) => Some(fc.path.as_str()),
            _ => None,
        })
        .collect();
    if let Err(e) = crate::git::stage::unstage_files(repo_path, &files) {
        model.popup = Some(PopupContent::Error {
            message: format!("Error unstaging files: {}", e),
        });
    }
    Some(Message::Refresh)
}
