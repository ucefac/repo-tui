//! Move target selector widget
//!
//! A modal widget for selecting the target main directory when moving a repository.

use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, List, ListItem, Paragraph, Widget},
};

/// Move target selector state
#[derive(Debug, Clone)]
pub struct MoveTargetSelectorState {
    /// Target directories to choose from
    pub targets: Vec<String>,
    /// Currently selected index
    pub selected_index: usize,
    /// Repository name being moved
    pub repo_name: String,
    /// Current main directory index (for skipping same directory)
    pub current_main_dir_index: Option<usize>,
}

impl MoveTargetSelectorState {
    /// Create a new move target selector state
    pub fn new(
        targets: Vec<String>,
        repo_name: String,
        current_main_dir_index: Option<usize>,
    ) -> Self {
        Self {
            targets,
            selected_index: 0,
            repo_name,
            current_main_dir_index,
        }
    }

    /// Move selection up
    pub fn select_previous(&mut self) {
        if self.targets.is_empty() {
            return;
        }
        self.selected_index = (self.selected_index + self.targets.len() - 1) % self.targets.len();
    }

    /// Move selection down
    pub fn select_next(&mut self) {
        if self.targets.is_empty() {
            return;
        }
        self.selected_index = (self.selected_index + 1) % self.targets.len();
    }

    /// Jump to first item
    pub fn select_first(&mut self) {
        if !self.targets.is_empty() {
            self.selected_index = 0;
        }
    }

    /// Jump to last item
    pub fn select_last(&mut self) {
        if !self.targets.is_empty() {
            self.selected_index = self.targets.len() - 1;
        }
    }

    /// Get the currently selected target
    pub fn selected(&self) -> Option<usize> {
        if self.targets.is_empty() {
            None
        } else {
            Some(self.selected_index)
        }
    }
}

/// Move target selector widget
pub struct MoveTargetSelector<'a> {
    state: &'a MoveTargetSelectorState,
}

impl<'a> MoveTargetSelector<'a> {
    /// Create a new move target selector
    pub fn new(state: &'a MoveTargetSelectorState) -> Self {
        Self { state }
    }
}

impl Widget for MoveTargetSelector<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Create outer block with border
        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::Rgb(129, 140, 248)))
            .title(" Move Repository ");

        let inner_area = block.inner(area);
        block.render(area, buf);

        // Create layout: title, list, hints
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),  // Repository info
                Constraint::Min(5),     // Target list
                Constraint::Length(2),  // Hints
            ])
            .split(inner_area);

        // Render repository info
        let repo_info = format!("Move '{}' to:", self.state.repo_name);
        let info_paragraph = Paragraph::new(repo_info)
            .style(Style::default().fg(Color::White))
            .alignment(Alignment::Center);
        info_paragraph.render(chunks[0], buf);

        // Render target list
        let items: Vec<ListItem> = self
            .state
            .targets
            .iter()
            .enumerate()
            .map(|(i, target)| {
                let is_current = self.state.current_main_dir_index == Some(i);
                let is_selected = i == self.state.selected_index;

                let prefix = if is_current {
                    "(current) "
                } else {
                    ""
                };

                let content = if is_selected {
                    Line::from(vec![
                        Span::styled(
                            format!("{}> {}", prefix, target),
                            Style::default()
                                .fg(Color::White)
                                .bg(Color::Rgb(79, 70, 229))
                                .add_modifier(Modifier::BOLD),
                        ),
                    ])
                } else if is_current {
                    Line::from(vec![
                        Span::styled(
                            format!("{}{}", prefix, target),
                            Style::default().fg(Color::Yellow),
                        ),
                    ])
                } else {
                    Line::from(vec![Span::raw(format!("  {}", target))])
                };

                ListItem::new(content)
            })
            .collect();

        let list = List::new(items);
        list.render(chunks[1], buf);

        // Render hints
        let hints = "↑↓ Navigate   Enter Confirm   Esc Cancel   Home/End Jump";
        let hints_paragraph = Paragraph::new(hints)
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center);
        hints_paragraph.render(chunks[2], buf);
    }
}

/// Calculate centered rectangle for move target selector
pub fn move_target_centered_rect(area: Rect, width_percent: u16, height_percent: u16) -> Rect {
    let popup_width = (area.width * width_percent) / 100;
    let popup_height = (area.height * height_percent) / 100;

    let x = (area.width - popup_width) / 2;
    let y = (area.height - popup_height) / 2;

    Rect::new(x, y, popup_width, popup_height)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_selector_state() {
        let targets = vec!["/home/user1".to_string(), "/home/user2".to_string()];
        let state = MoveTargetSelectorState::new(targets, "my-repo".to_string(), Some(0));

        assert_eq!(state.selected_index, 0);
        assert_eq!(state.repo_name, "my-repo");
        assert_eq!(state.current_main_dir_index, Some(0));
    }

    #[test]
    fn test_selector_wrap_around() {
        let targets = vec!["/home/user1".to_string(), "/home/user2".to_string()];
        let mut state = MoveTargetSelectorState::new(targets, "my-repo".to_string(), None);

        // Test next wraps around
        state.select_next();
        assert_eq!(state.selected_index, 1);
        state.select_next();
        assert_eq!(state.selected_index, 0);

        // Test previous wraps around
        state.select_previous();
        assert_eq!(state.selected_index, 1);
    }
}
