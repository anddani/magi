#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PopupContent {
    Error { message: String },
    Command(PopupContentCommand),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PopupContentCommand {
    Help,
    Commit,
}
