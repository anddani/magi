use std::path::PathBuf;

use ratatui::style::Color;
use serde::Deserialize;

use super::theme::Theme;

/// Parse a color string into a ratatui Color
/// Supports: named colors, hex (#ff0000, #f00), rgb (rgb(255, 0, 0)), indexed (0-255)
fn parse_color(s: &str) -> Option<Color> {
    let s = s.trim().to_lowercase();

    // Named colors
    match s.as_str() {
        "black" => return Some(Color::Black),
        "red" => return Some(Color::Red),
        "green" => return Some(Color::Green),
        "yellow" => return Some(Color::Yellow),
        "blue" => return Some(Color::Blue),
        "magenta" => return Some(Color::Magenta),
        "cyan" => return Some(Color::Cyan),
        "gray" | "grey" => return Some(Color::Gray),
        "darkgray" | "darkgrey" => return Some(Color::DarkGray),
        "lightred" => return Some(Color::LightRed),
        "lightgreen" => return Some(Color::LightGreen),
        "lightyellow" => return Some(Color::LightYellow),
        "lightblue" => return Some(Color::LightBlue),
        "lightmagenta" => return Some(Color::LightMagenta),
        "lightcyan" => return Some(Color::LightCyan),
        "white" => return Some(Color::White),
        "reset" => return Some(Color::Reset),
        _ => {}
    }

    // Hex color: #rrggbb or #rgb
    if let Some(hex) = s.strip_prefix('#') {
        if hex.len() == 6 {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            return Some(Color::Rgb(r, g, b));
        } else if hex.len() == 3 {
            let r = u8::from_str_radix(&hex[0..1], 16).ok()? * 17;
            let g = u8::from_str_radix(&hex[1..2], 16).ok()? * 17;
            let b = u8::from_str_radix(&hex[2..3], 16).ok()? * 17;
            return Some(Color::Rgb(r, g, b));
        }
    }

    // RGB format: rgb(r, g, b)
    if s.starts_with("rgb(") && s.ends_with(')') {
        let inner = &s[4..s.len() - 1];
        let parts: Vec<&str> = inner.split(',').collect();
        if parts.len() == 3 {
            let r: u8 = parts[0].trim().parse().ok()?;
            let g: u8 = parts[1].trim().parse().ok()?;
            let b: u8 = parts[2].trim().parse().ok()?;
            return Some(Color::Rgb(r, g, b));
        }
    }

    // ANSI 256 color: just a number
    if let Ok(n) = s.parse::<u8>() {
        return Some(Color::Indexed(n));
    }

    None
}

/// Whether the terminal has a dark or light background
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeMode {
    Dark,
    Light,
}

/// Color overrides in the config file
#[derive(Debug, Clone, Default, Deserialize)]
pub struct ColorOverrides {
    pub section_header: Option<String>,
    pub ref_label: Option<String>,
    pub tag_label: Option<String>,
    pub diff_addition: Option<String>,
    pub diff_deletion: Option<String>,
    pub diff_context: Option<String>,
    pub diff_hunk: Option<String>,
    pub remote_branch: Option<String>,
    pub local_branch: Option<String>,
    pub detached_head: Option<String>,
    pub untracked_file: Option<String>,
    pub unstaged_status: Option<String>,
    pub staged_status: Option<String>,
    pub file_path: Option<String>,
    pub commit_hash: Option<String>,
    pub text: Option<String>,
    pub dim_text: Option<String>,
    pub selection_bg: Option<String>,
    pub status_bar_bg: Option<String>,
    pub status_bar_fg: Option<String>,
    pub status_mode_normal_bg: Option<String>,
    pub status_mode_normal_fg: Option<String>,
    pub status_mode_visual_bg: Option<String>,
    pub status_mode_visual_fg: Option<String>,
    pub status_mode_search_bg: Option<String>,
    pub status_mode_search_fg: Option<String>,
    pub search_match_bg: Option<String>,
    pub search_match_fg: Option<String>,
}

/// Main config structure
#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    #[serde(default = "default_theme_name")]
    pub theme: String,

    /// Theme picked when `theme = "auto"` and the terminal is dark
    #[serde(default)]
    pub theme_dark: Option<String>,

    /// Theme picked when `theme = "auto"` and the terminal is light
    #[serde(default)]
    pub theme_light: Option<String>,

    #[serde(default)]
    pub colors: ColorOverrides,

    #[serde(default)]
    pub language: Option<String>,
}

fn default_theme_name() -> String {
    "auto".to_string()
}

impl Default for Config {
    fn default() -> Self {
        Self {
            theme: default_theme_name(),
            theme_dark: None,
            theme_light: None,
            colors: ColorOverrides::default(),
            language: None,
        }
    }
}

impl Config {
    /// Get the default config file path
    pub fn default_path() -> Option<PathBuf> {
        dirs::config_dir().map(|p| p.join("magi").join("config.toml"))
    }

    /// Load config from the default path, or return default config
    pub fn load() -> Self {
        Self::default_path()
            .and_then(|path| Self::load_from_path(&path).ok())
            .unwrap_or_default()
    }

    /// Load config from a specific path
    pub fn load_from_path(path: &PathBuf) -> Result<Self, ConfigError> {
        let contents =
            std::fs::read_to_string(path).map_err(|e| ConfigError::IoError(e.to_string()))?;
        toml::from_str(&contents).map_err(|e| ConfigError::ParseError(e.to_string()))
    }

    /// Whether the theme should be picked from the detected terminal background
    pub fn is_auto_theme(&self) -> bool {
        self.theme.eq_ignore_ascii_case("auto")
    }

    /// Resolve the theme with overrides applied.
    ///
    /// `detected` is the terminal background mode detected at startup. It is
    /// only consulted when `theme = "auto"`; detection failure (`None`) falls
    /// back to the dark theme.
    pub fn resolve_theme(&self, detected: Option<ThemeMode>) -> Theme {
        let name: &str = if self.is_auto_theme() {
            match detected.unwrap_or(ThemeMode::Dark) {
                ThemeMode::Dark => self.theme_dark.as_deref().unwrap_or("default"),
                ThemeMode::Light => self.theme_light.as_deref().unwrap_or("default-light"),
            }
        } else {
            &self.theme
        };
        let mut theme = Theme::from_name(name).unwrap_or_else(Theme::default_theme);

        // Apply overrides using a macro to reduce repetition
        macro_rules! apply_overrides {
            ($($field:ident),+ $(,)?) => {
                $(
                    if let Some(ref color_str) = self.colors.$field {
                        if let Some(color) = parse_color(color_str) {
                            theme.$field = color;
                        }
                    }
                )+
            };
        }

        apply_overrides!(
            section_header,
            ref_label,
            tag_label,
            diff_addition,
            diff_deletion,
            diff_context,
            diff_hunk,
            remote_branch,
            local_branch,
            detached_head,
            untracked_file,
            unstaged_status,
            staged_status,
            file_path,
            commit_hash,
            text,
            dim_text,
            selection_bg,
            status_bar_bg,
            status_bar_fg,
            status_mode_normal_bg,
            status_mode_normal_fg,
            status_mode_visual_bg,
            status_mode_visual_fg,
            status_mode_search_bg,
            status_mode_search_fg,
            search_match_bg,
            search_match_fg,
        );

        theme
    }
}

#[derive(Debug)]
pub enum ConfigError {
    IoError(String),
    ParseError(String),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::IoError(e) => write!(f, "IO error: {}", e),
            ConfigError::ParseError(e) => write!(f, "Parse error: {}", e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_hex_color() {
        assert_eq!(parse_color("#ff0000"), Some(Color::Rgb(255, 0, 0)));
        assert_eq!(parse_color("#f00"), Some(Color::Rgb(255, 0, 0)));
        assert_eq!(parse_color("#00ff00"), Some(Color::Rgb(0, 255, 0)));
    }

    #[test]
    fn test_parse_named_color() {
        assert_eq!(parse_color("red"), Some(Color::Red));
        assert_eq!(parse_color("Green"), Some(Color::Green));
        assert_eq!(parse_color("BLUE"), Some(Color::Blue));
    }

    #[test]
    fn test_parse_rgb_color() {
        assert_eq!(
            parse_color("rgb(100, 150, 200)"),
            Some(Color::Rgb(100, 150, 200))
        );
        assert_eq!(parse_color("rgb(0,0,0)"), Some(Color::Rgb(0, 0, 0)));
    }

    #[test]
    fn test_parse_indexed_color() {
        assert_eq!(parse_color("196"), Some(Color::Indexed(196)));
    }

    #[test]
    fn test_config_with_overrides() {
        let toml_str = r##"
            theme = "default"
            [colors]
            section_header = "#ff0000"
        "##;
        let config: Config = toml::from_str(toml_str).unwrap();
        let theme = config.resolve_theme(None);
        assert_eq!(theme.section_header, Color::Rgb(255, 0, 0));
        // Other colors should remain default
        assert_eq!(theme.diff_addition, Color::Green);
    }

    #[test]
    fn test_config_with_theme() {
        let toml_str = r#"
            theme = "catppuccin-frappe"
        "#;
        let config: Config = toml::from_str(toml_str).unwrap();
        let theme = config.resolve_theme(None);
        assert_eq!(theme.section_header, Color::Rgb(229, 200, 144));
    }

    #[test]
    fn test_default_config() {
        let config = Config::default();
        // Default theme is "auto"; failed detection falls back to dark default
        assert!(config.is_auto_theme());
        let theme = config.resolve_theme(None);
        assert_eq!(theme.section_header, Color::Yellow);
    }

    #[test]
    fn test_auto_theme_picks_light() {
        let config = Config::default();
        let theme = config.resolve_theme(Some(ThemeMode::Light));
        assert_eq!(theme, Theme::default_light());
    }

    #[test]
    fn test_auto_theme_picks_dark() {
        let config = Config::default();
        let theme = config.resolve_theme(Some(ThemeMode::Dark));
        assert_eq!(theme, Theme::default_theme());
    }

    #[test]
    fn test_auto_theme_respects_theme_dark_and_theme_light() {
        let toml_str = r#"
            theme = "auto"
            theme_dark = "catppuccin-mocha"
            theme_light = "catppuccin-latte"
        "#;
        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(
            config.resolve_theme(Some(ThemeMode::Dark)),
            Theme::catppuccin_mocha()
        );
        assert_eq!(
            config.resolve_theme(Some(ThemeMode::Light)),
            Theme::catppuccin_latte()
        );
    }

    #[test]
    fn test_explicit_theme_ignores_detected_mode() {
        let toml_str = r#"
            theme = "catppuccin-mocha"
        "#;
        let config: Config = toml::from_str(toml_str).unwrap();
        assert!(!config.is_auto_theme());
        let theme = config.resolve_theme(Some(ThemeMode::Light));
        assert_eq!(theme, Theme::catppuccin_mocha());
    }

    #[test]
    fn test_is_auto_theme_case_insensitive() {
        let config = Config {
            theme: "Auto".to_string(),
            ..Config::default()
        };
        assert!(config.is_auto_theme());
    }

    #[test]
    fn test_unknown_theme_dark_falls_back_to_default() {
        let toml_str = r#"
            theme = "auto"
            theme_dark = "nonexistent"
        "#;
        let config: Config = toml::from_str(toml_str).unwrap();
        let theme = config.resolve_theme(Some(ThemeMode::Dark));
        assert_eq!(theme, Theme::default_theme());
    }

    #[test]
    fn test_new_color_overrides() {
        let toml_str = r##"
            theme = "default"
            [colors]
            dim_text = "#808080"
            search_match_bg = "#ffff00"
            status_mode_normal_bg = "blue"
            staged_status = "cyan"
        "##;
        let config: Config = toml::from_str(toml_str).unwrap();
        let theme = config.resolve_theme(None);
        assert_eq!(theme.dim_text, Color::Rgb(128, 128, 128));
        assert_eq!(theme.search_match_bg, Color::Rgb(255, 255, 0));
        assert_eq!(theme.status_mode_normal_bg, Color::Blue);
        assert_eq!(theme.staged_status, Color::Cyan);
    }

    #[test]
    fn test_overrides_apply_on_auto_resolved_theme() {
        let toml_str = r##"
            theme = "auto"
            [colors]
            dim_text = "#333333"
        "##;
        let config: Config = toml::from_str(toml_str).unwrap();
        let theme = config.resolve_theme(Some(ThemeMode::Light));
        assert_eq!(theme.dim_text, Color::Rgb(51, 51, 51));
        // Rest of the theme is default-light
        assert_eq!(theme.selection_bg, Theme::default_light().selection_bg);
    }
}
