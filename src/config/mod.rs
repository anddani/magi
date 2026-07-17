mod detect;
mod settings;
mod theme;

pub use detect::detect_theme_mode;
pub use settings::{Config, ThemeMode};
pub use theme::Theme;
