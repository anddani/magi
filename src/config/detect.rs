use terminal_colorsaurus::{QueryOptions, ThemeMode as ColorsaurusThemeMode, theme_mode};

use super::settings::ThemeMode;

/// Query the terminal background color (OSC 11) to determine whether the
/// terminal is dark or light.
///
/// Must be called before Ratatui enters raw mode / the alternate screen,
/// otherwise the terminal's response can be swallowed by crossterm's event
/// reads. Returns `None` when detection is unsupported or fails (piped
/// output, `TERM=dumb`, terminals without OSC 11 support).
pub fn detect_theme_mode() -> Option<ThemeMode> {
    match theme_mode(QueryOptions::default()) {
        Ok(ColorsaurusThemeMode::Dark) => Some(ThemeMode::Dark),
        Ok(ColorsaurusThemeMode::Light) => Some(ThemeMode::Light),
        Err(_) => None,
    }
}
