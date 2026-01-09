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
    if s.starts_with('#') {
        let hex = &s[1..];
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
    pub file_path: Option<String>,
    pub commit_hash: Option<String>,
    pub text: Option<String>,
    pub selection_bg: Option<String>,
}

/// Main config structure
#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    #[serde(default = "default_theme_name")]
    pub theme: String,

    #[serde(default)]
    pub colors: ColorOverrides,
}

fn default_theme_name() -> String {
    "default".to_string()
}

impl Default for Config {
    fn default() -> Self {
        Self {
            theme: default_theme_name(),
            colors: ColorOverrides::default(),
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

    /// Resolve the theme with overrides applied
    pub fn resolve_theme(&self) -> Theme {
        let mut theme = Theme::from_name(&self.theme).unwrap_or_else(Theme::default_theme);

        // Apply overrides using a macro to reduce repetition
        macro_rules! apply_override {
            ($field:ident) => {
                if let Some(ref color_str) = self.colors.$field {
                    if let Some(color) = parse_color(color_str) {
                        theme.$field = color;
                    }
                }
            };
        }

        apply_override!(section_header);
        apply_override!(ref_label);
        apply_override!(tag_label);
        apply_override!(diff_addition);
        apply_override!(diff_deletion);
        apply_override!(diff_context);
        apply_override!(diff_hunk);
        apply_override!(remote_branch);
        apply_override!(local_branch);
        apply_override!(detached_head);
        apply_override!(untracked_file);
        apply_override!(unstaged_status);
        apply_override!(file_path);
        apply_override!(commit_hash);
        apply_override!(text);
        apply_override!(selection_bg);

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
        let theme = config.resolve_theme();
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
        let theme = config.resolve_theme();
        assert_eq!(theme.section_header, Color::Rgb(229, 200, 144));
    }

    #[test]
    fn test_default_config() {
        let config = Config::default();
        let theme = config.resolve_theme();
        assert_eq!(theme.section_header, Color::Yellow);
    }
}
