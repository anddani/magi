use crate::{
    git::file_checkout::file_checkout,
    model::{Model, popup::PopupContent},
    msg::Message,
};

pub fn update(model: &mut Model, revision: String, file: String) -> Option<Message> {
    model.popup = None;

    match file_checkout(&model.workdir, &revision, &file) {
        Ok(Ok(())) => Some(Message::Refresh),
        Ok(Err(err)) => {
            model.popup = Some(PopupContent::Error {
                message: format!("Failed to checkout {} from {}: {}", file, revision, err),
            });
            None
        }
        Err(err) => {
            model.popup = Some(PopupContent::Error {
                message: format!("Failed to checkout {} from {}: {}", file, revision, err),
            });
            None
        }
    }
}
