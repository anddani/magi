use ratatui::style::Color;

/// Represents all semantic color roles in the application
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Theme {
    // Section and label colors
    pub section_header: Color,
    pub ref_label: Color,
    pub tag_label: Color,

    // Diff colors
    pub diff_addition: Color,
    pub diff_deletion: Color,
    pub diff_context: Color,
    pub diff_hunk: Color,

    // Branch colors
    pub remote_branch: Color,
    pub local_branch: Color,
    pub detached_head: Color,

    // File colors
    pub untracked_file: Color,
    pub unstaged_status: Color,
    pub staged_status: Color,
    pub file_path: Color,

    // Misc
    pub commit_hash: Color,
    pub text: Color,
    /// Dim/faded text such as popup hints, argument descriptions and
    /// log author/time. Must be readable on the theme's background.
    pub dim_text: Color,

    // Selection
    pub selection_bg: Color,

    // Status bar
    pub status_bar_bg: Color,
    pub status_bar_fg: Color,
    pub status_mode_normal_bg: Color,
    pub status_mode_normal_fg: Color,
    pub status_mode_visual_bg: Color,
    pub status_mode_visual_fg: Color,
    pub status_mode_search_bg: Color,
    pub status_mode_search_fg: Color,

    // Search match highlight
    pub search_match_bg: Color,
    pub search_match_fg: Color,
}

impl Default for Theme {
    fn default() -> Self {
        Self::default_theme()
    }
}

impl Theme {
    /// The default theme matching current hardcoded colors
    pub fn default_theme() -> Self {
        Self {
            section_header: Color::Yellow,
            ref_label: Color::Yellow,
            tag_label: Color::Yellow,
            diff_addition: Color::Green,
            diff_deletion: Color::Red,
            diff_context: Color::Reset,
            diff_hunk: Color::Cyan,
            remote_branch: Color::Green,
            local_branch: Color::Blue,
            detached_head: Color::Red,
            untracked_file: Color::Red,
            unstaged_status: Color::Magenta,
            staged_status: Color::Green,
            file_path: Color::Reset,
            commit_hash: Color::Rgb(139, 69, 19),
            text: Color::Reset,
            dim_text: Color::DarkGray,
            selection_bg: Color::Rgb(60, 60, 80),
            status_bar_bg: Color::Rgb(40, 40, 50),
            status_bar_fg: Color::Reset,
            status_mode_normal_bg: Color::Rgb(100, 149, 237), // Cornflower blue
            status_mode_normal_fg: Color::Rgb(30, 30, 40),
            status_mode_visual_bg: Color::Rgb(186, 133, 217), // Bright purple
            status_mode_visual_fg: Color::Rgb(30, 30, 40),
            status_mode_search_bg: Color::Rgb(250, 215, 140), // Warm yellow
            status_mode_search_fg: Color::Rgb(30, 30, 40),
            search_match_bg: Color::Rgb(250, 215, 140), // Warm yellow
            search_match_fg: Color::Rgb(30, 30, 40),
        }
    }

    /// Light counterpart to the default theme. Uses the terminal's default
    /// foreground where possible and dark, readable accents elsewhere.
    pub fn default_light() -> Self {
        Self {
            section_header: Color::Rgb(176, 121, 0), // Dark amber
            ref_label: Color::Rgb(176, 121, 0),
            tag_label: Color::Rgb(176, 121, 0),
            diff_addition: Color::Rgb(0, 128, 0),   // Dark green
            diff_deletion: Color::Rgb(200, 30, 45), // Dark red
            diff_context: Color::Reset,
            diff_hunk: Color::Rgb(0, 120, 135), // Dark teal
            remote_branch: Color::Rgb(0, 128, 0),
            local_branch: Color::Rgb(20, 80, 200), // Readable blue
            detached_head: Color::Rgb(200, 30, 45),
            untracked_file: Color::Rgb(200, 30, 45),
            unstaged_status: Color::Rgb(160, 50, 160), // Dark magenta
            staged_status: Color::Rgb(0, 128, 0),
            file_path: Color::Reset,
            commit_hash: Color::Rgb(139, 69, 19), // Saddle brown
            text: Color::Reset,
            dim_text: Color::Rgb(115, 115, 115),     // Mid gray
            selection_bg: Color::Rgb(220, 225, 240), // Light blue-gray
            status_bar_bg: Color::Rgb(225, 228, 235),
            status_bar_fg: Color::Rgb(40, 40, 50),
            // Saturated pill backgrounds with dark foregrounds read on
            // both light and dark terminals, so these match the default theme.
            status_mode_normal_bg: Color::Rgb(100, 149, 237), // Cornflower blue
            status_mode_normal_fg: Color::Rgb(30, 30, 40),
            status_mode_visual_bg: Color::Rgb(186, 133, 217), // Bright purple
            status_mode_visual_fg: Color::Rgb(30, 30, 40),
            status_mode_search_bg: Color::Rgb(250, 215, 140), // Warm yellow
            status_mode_search_fg: Color::Rgb(30, 30, 40),
            search_match_bg: Color::Rgb(250, 215, 140), // Warm yellow
            search_match_fg: Color::Rgb(30, 30, 40),
        }
    }

    /// Catppuccin Frappe theme
    pub fn catppuccin_frappe() -> Self {
        Self {
            section_header: Color::Rgb(229, 200, 144),        // Yellow
            ref_label: Color::Rgb(229, 200, 144),             // Yellow
            tag_label: Color::Rgb(229, 200, 144),             // Yellow
            diff_addition: Color::Rgb(166, 209, 137),         // Green
            diff_deletion: Color::Rgb(231, 130, 132),         // Red
            diff_context: Color::Rgb(198, 208, 245),          // Text
            diff_hunk: Color::Rgb(140, 170, 238),             // Blue
            remote_branch: Color::Rgb(166, 209, 137),         // Green
            local_branch: Color::Rgb(140, 170, 238),          // Blue
            detached_head: Color::Rgb(231, 130, 132),         // Red
            untracked_file: Color::Rgb(231, 130, 132),        // Red
            unstaged_status: Color::Rgb(244, 184, 228),       // Pink
            staged_status: Color::Rgb(166, 209, 137),         // Green
            file_path: Color::Rgb(198, 208, 245),             // Text
            commit_hash: Color::Rgb(239, 159, 118),           // Peach
            text: Color::Rgb(198, 208, 245),                  // Text
            dim_text: Color::Rgb(131, 139, 167),              // Overlay1
            selection_bg: Color::Rgb(65, 69, 89),             // Surface0
            status_bar_bg: Color::Rgb(48, 52, 70),            // Surface0
            status_bar_fg: Color::Rgb(198, 208, 245),         // Text
            status_mode_normal_bg: Color::Rgb(140, 170, 238), // Blue
            status_mode_normal_fg: Color::Rgb(48, 52, 70),    // Surface0
            status_mode_visual_bg: Color::Rgb(202, 158, 230), // Mauve
            status_mode_visual_fg: Color::Rgb(48, 52, 70),    // Surface0
            status_mode_search_bg: Color::Rgb(229, 200, 144), // Yellow
            status_mode_search_fg: Color::Rgb(48, 52, 70),    // Surface0
            search_match_bg: Color::Rgb(229, 200, 144),       // Yellow
            search_match_fg: Color::Rgb(48, 52, 70),          // Surface0
        }
    }

    /// Catppuccin Mocha theme
    pub fn catppuccin_mocha() -> Self {
        Self {
            section_header: Color::Rgb(249, 226, 175),        // Yellow
            ref_label: Color::Rgb(249, 226, 175),             // Yellow
            tag_label: Color::Rgb(249, 226, 175),             // Yellow
            diff_addition: Color::Rgb(166, 227, 161),         // Green
            diff_deletion: Color::Rgb(243, 139, 168),         // Red
            diff_context: Color::Rgb(205, 214, 244),          // Text
            diff_hunk: Color::Rgb(137, 180, 250),             // Blue
            remote_branch: Color::Rgb(166, 227, 161),         // Green
            local_branch: Color::Rgb(137, 180, 250),          // Blue
            detached_head: Color::Rgb(243, 139, 168),         // Red
            untracked_file: Color::Rgb(243, 139, 168),        // Red
            unstaged_status: Color::Rgb(245, 194, 231),       // Pink
            staged_status: Color::Rgb(166, 227, 161),         // Green
            file_path: Color::Rgb(205, 214, 244),             // Text
            commit_hash: Color::Rgb(250, 179, 135),           // Peach
            text: Color::Rgb(205, 214, 244),                  // Text
            dim_text: Color::Rgb(127, 132, 156),              // Overlay1
            selection_bg: Color::Rgb(49, 50, 68),             // Surface0
            status_bar_bg: Color::Rgb(30, 30, 46),            // Base
            status_bar_fg: Color::Rgb(205, 214, 244),         // Text
            status_mode_normal_bg: Color::Rgb(137, 180, 250), // Blue
            status_mode_normal_fg: Color::Rgb(30, 30, 46),    // Base
            status_mode_visual_bg: Color::Rgb(203, 166, 247), // Mauve
            status_mode_visual_fg: Color::Rgb(30, 30, 46),    // Base
            status_mode_search_bg: Color::Rgb(249, 226, 175), // Yellow
            status_mode_search_fg: Color::Rgb(30, 30, 46),    // Base
            search_match_bg: Color::Rgb(249, 226, 175),       // Yellow
            search_match_fg: Color::Rgb(30, 30, 46),          // Base
        }
    }

    /// Catppuccin Latte theme (light)
    pub fn catppuccin_latte() -> Self {
        Self {
            section_header: Color::Rgb(223, 142, 29),   // Yellow
            ref_label: Color::Rgb(223, 142, 29),        // Yellow
            tag_label: Color::Rgb(223, 142, 29),        // Yellow
            diff_addition: Color::Rgb(64, 160, 43),     // Green
            diff_deletion: Color::Rgb(210, 15, 57),     // Red
            diff_context: Color::Rgb(76, 79, 105),      // Text
            diff_hunk: Color::Rgb(30, 102, 245),        // Blue
            remote_branch: Color::Rgb(64, 160, 43),     // Green
            local_branch: Color::Rgb(30, 102, 245),     // Blue
            detached_head: Color::Rgb(210, 15, 57),     // Red
            untracked_file: Color::Rgb(210, 15, 57),    // Red
            unstaged_status: Color::Rgb(234, 118, 203), // Pink
            staged_status: Color::Rgb(64, 160, 43),     // Green
            file_path: Color::Rgb(76, 79, 105),         // Text
            commit_hash: Color::Rgb(254, 100, 11),      // Peach
            text: Color::Rgb(76, 79, 105),              // Text
            dim_text: Color::Rgb(140, 143, 161),        // Overlay1
            selection_bg: Color::Rgb(204, 208, 218),    // Surface0
            status_bar_bg: Color::Rgb(230, 233, 239),   // Mantle
            status_bar_fg: Color::Rgb(76, 79, 105),     // Text
            // Latte accents are dark, so pills use light Base as foreground
            // (the inverse of Frappe/Mocha).
            status_mode_normal_bg: Color::Rgb(30, 102, 245), // Blue
            status_mode_normal_fg: Color::Rgb(239, 241, 245), // Base
            status_mode_visual_bg: Color::Rgb(136, 57, 239), // Mauve
            status_mode_visual_fg: Color::Rgb(239, 241, 245), // Base
            status_mode_search_bg: Color::Rgb(223, 142, 29), // Yellow
            status_mode_search_fg: Color::Rgb(239, 241, 245), // Base
            search_match_bg: Color::Rgb(223, 142, 29),       // Yellow
            search_match_fg: Color::Rgb(239, 241, 245),      // Base
        }
    }

    /// Get a built-in theme by name. Note that "auto" is not a theme name;
    /// it is resolved to a concrete theme in `Config::resolve_theme`.
    pub fn from_name(name: &str) -> Option<Self> {
        match name.to_lowercase().replace('_', "-").as_str() {
            "default" => Some(Self::default_theme()),
            "default-light" => Some(Self::default_light()),
            "catppuccin-frappe" => Some(Self::catppuccin_frappe()),
            "catppuccin-mocha" => Some(Self::catppuccin_mocha()),
            "catppuccin-latte" => Some(Self::catppuccin_latte()),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_theme() {
        let theme = Theme::default();
        assert_eq!(theme.section_header, Color::Yellow);
        assert_eq!(theme.diff_addition, Color::Green);
        assert_eq!(theme.dim_text, Color::DarkGray);
        // Text-like fields use the terminal's default foreground so the
        // default theme adapts to the terminal palette.
        assert_eq!(theme.diff_context, Color::Reset);
        assert_eq!(theme.file_path, Color::Reset);
    }

    #[test]
    fn test_from_name() {
        assert!(Theme::from_name("default").is_some());
        assert!(Theme::from_name("default-light").is_some());
        assert!(Theme::from_name("default_light").is_some());
        assert!(Theme::from_name("catppuccin-frappe").is_some());
        assert!(Theme::from_name("catppuccin_frappe").is_some());
        assert!(Theme::from_name("catppuccin-mocha").is_some());
        assert!(Theme::from_name("catppuccin-latte").is_some());
        assert!(Theme::from_name("catppuccin_latte").is_some());
        assert!(Theme::from_name("nonexistent").is_none());
        // "auto" is resolved in Config, not a theme name
        assert!(Theme::from_name("auto").is_none());
    }

    #[test]
    fn test_catppuccin_frappe_colors() {
        let theme = Theme::catppuccin_frappe();
        assert_eq!(theme.section_header, Color::Rgb(229, 200, 144));
        assert_eq!(theme.diff_addition, Color::Rgb(166, 209, 137));
        assert_eq!(theme.dim_text, Color::Rgb(131, 139, 167));
    }

    #[test]
    fn test_default_light_colors() {
        let theme = Theme::default_light();
        assert_eq!(theme.text, Color::Reset);
        assert_eq!(theme.selection_bg, Color::Rgb(220, 225, 240));
        assert_eq!(theme.dim_text, Color::Rgb(115, 115, 115));
    }

    #[test]
    fn test_catppuccin_latte_colors() {
        let theme = Theme::catppuccin_latte();
        assert_eq!(theme.section_header, Color::Rgb(223, 142, 29));
        assert_eq!(theme.diff_addition, Color::Rgb(64, 160, 43));
        assert_eq!(theme.text, Color::Rgb(76, 79, 105));
        assert_eq!(theme.dim_text, Color::Rgb(140, 143, 161));
    }
}
