use crate::{
    model::{
        Model,
        arguments::{Arguments::LogArguments, LogArgument},
        popup::{PopupContent, PopupContentCommand},
    },
    msg::Message,
};

pub fn update(model: &mut Model) -> Option<Message> {
    // Show graph is enabled by default
    model.arguments = Some(LogArguments([LogArgument::Graph].into_iter().collect()));
    model.popup = Some(PopupContent::Command(PopupContentCommand::Log));
    None
}
