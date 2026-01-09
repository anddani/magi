use ratatui::style::Color;

/// Represents all semantic color roles in the application
#[derive(Debug, Clone)]
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

    // Selection
    pub selection_bg: Color,
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
            diff_context: Color::White,
            diff_hunk: Color::Cyan,
            remote_branch: Color::Green,
            local_branch: Color::Blue,
            detached_head: Color::Red,
            untracked_file: Color::Red,
            unstaged_status: Color::Magenta,
            staged_status: Color::Green,
            file_path: Color::White,
            commit_hash: Color::Rgb(139, 69, 19),
            text: Color::Reset,
            selection_bg: Color::Rgb(60, 60, 80),
        }
    }

    /// Catppuccin Frappe theme
    pub fn catppuccin_frappe() -> Self {
        Self {
            section_header: Color::Rgb(229, 200, 144),  // Yellow
            ref_label: Color::Rgb(229, 200, 144),       // Yellow
            tag_label: Color::Rgb(229, 200, 144),       // Yellow
            diff_addition: Color::Rgb(166, 209, 137),   // Green
            diff_deletion: Color::Rgb(231, 130, 132),   // Red
            diff_context: Color::Rgb(198, 208, 245),    // Text
            diff_hunk: Color::Rgb(140, 170, 238),       // Blue
            remote_branch: Color::Rgb(166, 209, 137),   // Green
            local_branch: Color::Rgb(140, 170, 238),    // Blue
            detached_head: Color::Rgb(231, 130, 132),   // Red
            untracked_file: Color::Rgb(231, 130, 132),  // Red
            unstaged_status: Color::Rgb(244, 184, 228), // Pink
            staged_status: Color::Rgb(166, 209, 137),   // Green
            file_path: Color::Rgb(198, 208, 245),       // Text
            commit_hash: Color::Rgb(239, 159, 118),     // Peach
            text: Color::Rgb(198, 208, 245),            // Text
            selection_bg: Color::Rgb(65, 69, 89),       // Surface0
        }
    }

    /// Catppuccin Mocha theme
    pub fn catppuccin_mocha() -> Self {
        Self {
            section_header: Color::Rgb(249, 226, 175),  // Yellow
            ref_label: Color::Rgb(249, 226, 175),       // Yellow
            tag_label: Color::Rgb(249, 226, 175),       // Yellow
            diff_addition: Color::Rgb(166, 227, 161),   // Green
            diff_deletion: Color::Rgb(243, 139, 168),   // Red
            diff_context: Color::Rgb(205, 214, 244),    // Text
            diff_hunk: Color::Rgb(137, 180, 250),       // Blue
            remote_branch: Color::Rgb(166, 227, 161),   // Green
            local_branch: Color::Rgb(137, 180, 250),    // Blue
            detached_head: Color::Rgb(243, 139, 168),   // Red
            untracked_file: Color::Rgb(243, 139, 168),  // Red
            unstaged_status: Color::Rgb(245, 194, 231), // Pink
            staged_status: Color::Rgb(166, 227, 161),   // Green
            file_path: Color::Rgb(205, 214, 244),       // Text
            commit_hash: Color::Rgb(250, 179, 135),     // Peach
            text: Color::Rgb(205, 214, 244),            // Text
            selection_bg: Color::Rgb(49, 50, 68),       // Surface0
        }
    }

    /// Get a built-in theme by name
    pub fn from_name(name: &str) -> Option<Self> {
        match name.to_lowercase().replace('_', "-").as_str() {
            "default" => Some(Self::default_theme()),
            "catppuccin-frappe" => Some(Self::catppuccin_frappe()),
            "catppuccin-mocha" => Some(Self::catppuccin_mocha()),
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
    }

    #[test]
    fn test_from_name() {
        assert!(Theme::from_name("default").is_some());
        assert!(Theme::from_name("catppuccin-frappe").is_some());
        assert!(Theme::from_name("catppuccin_frappe").is_some());
        assert!(Theme::from_name("catppuccin-mocha").is_some());
        assert!(Theme::from_name("nonexistent").is_none());
    }

    #[test]
    fn test_catppuccin_frappe_colors() {
        let theme = Theme::catppuccin_frappe();
        assert_eq!(theme.section_header, Color::Rgb(229, 200, 144));
        assert_eq!(theme.diff_addition, Color::Rgb(166, 209, 137));
    }
}
