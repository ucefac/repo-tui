//! Responsive layout utilities
//!
//! Provides layout calculation and responsive breakpoints for adaptive UI.

use ratatui::layout::{Constraint, Rect};

/// Responsive breakpoints
pub const WIDTH_SM: u16 = 60; // Small: hide metadata
pub const WIDTH_MD: u16 = 100; // Medium: show branch
pub const WIDTH_LG: u16 = 120; // Large: full info

/// Minimum terminal dimensions
pub const MIN_WIDTH: u16 = 80;
pub const MIN_HEIGHT: u16 = 25;

/// Calculate main layout chunks
///
/// Returns (title_bar, search_box, repo_list, status_bar)
pub fn calculate_main_layout(area: Rect) -> (Rect, Rect, Rect, Rect) {
    use ratatui::layout::{Direction, Layout};

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Title bar
            Constraint::Length(3), // Search box
            Constraint::Min(10),   // Repository list
            Constraint::Length(4), // Status bar (status + path)
        ])
        .split(area);

    (chunks[0], chunks[1], chunks[2], chunks[3])
}

/// Calculate repository list row constraints based on terminal width
pub fn calculate_repo_list_row(width: u16) -> Vec<Constraint> {
    if width < WIDTH_SM {
        // Small: only repo name
        vec![Constraint::Percentage(100)]
    } else if width < WIDTH_MD {
        // Medium: repo name + branch
        vec![
            Constraint::Percentage(60), // Repo name
            Constraint::Percentage(40), // Branch
        ]
    } else if width < WIDTH_LG {
        // Large: repo name + branch + status
        vec![
            Constraint::Percentage(50), // Repo name
            Constraint::Percentage(25), // Branch
            Constraint::Percentage(25), // Status
        ]
    } else {
        // Extra large: full info
        vec![
            Constraint::Percentage(45), // Repo name
            Constraint::Percentage(20), // Branch
            Constraint::Percentage(15), // Status
            Constraint::Percentage(20), // Path
        ]
    }
}

/// Truncate text from the middle with ellipsis
///
/// Preserves both start and end of text: "very-long-path" → "very-...-path"
pub fn truncate_middle(text: &str, max_len: usize) -> String {
    if text.len() <= max_len {
        return text.to_string();
    }

    if max_len < 5 {
        return text[..max_len.min(text.len())].to_string();
    }

    let available = max_len - 3; // Space for "..."
    let start_len = available / 2;
    let end_len = available - start_len;

    format!("{}...{}", &text[..start_len], &text[text.len() - end_len..])
}

/// Get display mode based on terminal width
pub fn get_display_mode(width: u16) -> DisplayMode {
    if width < WIDTH_SM {
        DisplayMode::Compact
    } else if width < WIDTH_MD {
        DisplayMode::Medium
    } else if width < WIDTH_LG {
        DisplayMode::Large
    } else {
        DisplayMode::ExtraLarge
    }
}

/// Display mode for responsive rendering
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplayMode {
    /// Compact: only essential info
    Compact,
    /// Medium: essential + branch
    Medium,
    /// Large: essential + branch + status
    Large,
    /// Extra large: full info
    ExtraLarge,
}

impl DisplayMode {
    /// Check if should show branch
    pub fn show_branch(self) -> bool {
        matches!(self, Self::Medium | Self::Large | Self::ExtraLarge)
    }

    /// Check if should show status
    pub fn show_status(self) -> bool {
        matches!(self, Self::Large | Self::ExtraLarge)
    }

    /// Check if should show path
    pub fn show_path(self) -> bool {
        matches!(self, Self::ExtraLarge)
    }

    /// Get max repo name length
    pub fn max_name_length(self) -> usize {
        match self {
            Self::Compact => 40,
            Self::Medium => 48,
            Self::Large => 38,
            Self::ExtraLarge => 35,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_truncate_middle() {
        assert_eq!(truncate_middle("hello", 10), "hello");
        assert_eq!(truncate_middle("hello-world", 10), "hel...orld");
        assert_eq!(truncate_middle("very-long-text", 8), "ve...ext");
    }

    #[test]
    fn test_truncate_middle_short() {
        assert_eq!(truncate_middle("hello", 3), "hel");
        assert_eq!(truncate_middle("hi", 3), "hi");
    }

    #[test]
    fn test_get_display_mode() {
        assert_eq!(get_display_mode(50), DisplayMode::Compact);
        assert_eq!(get_display_mode(80), DisplayMode::Medium);
        assert_eq!(get_display_mode(110), DisplayMode::Large);
        assert_eq!(get_display_mode(130), DisplayMode::ExtraLarge);
    }

    #[test]
    fn test_display_mode_features() {
        assert!(!DisplayMode::Compact.show_branch());
        assert!(DisplayMode::Medium.show_branch());
        assert!(!DisplayMode::Compact.show_status());
        assert!(DisplayMode::Large.show_status());
        assert!(!DisplayMode::Large.show_path());
        assert!(DisplayMode::ExtraLarge.show_path());
    }

    #[test]
    fn test_calculate_repo_list_row() {
        let small = calculate_repo_list_row(50);
        assert_eq!(small.len(), 1);

        let medium = calculate_repo_list_row(80);
        assert_eq!(medium.len(), 2);

        let large = calculate_repo_list_row(110);
        assert_eq!(large.len(), 3);

        let xl = calculate_repo_list_row(130);
        assert_eq!(xl.len(), 4);
    }
}
