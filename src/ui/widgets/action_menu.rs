//! Action menu widget
//!
//! Popup menu for selecting actions on a repository.

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    prelude::*,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem},
    Frame,
};

use crate::action::Action;
use crate::repo::Repository;

/// Action menu widget
pub struct ActionMenu<'a> {
    repo: &'a Repository,
    selected_index: usize,
}

impl<'a> ActionMenu<'a> {
    /// Create a new action menu
    pub fn new(repo: &'a Repository, selected_index: usize) -> Self {
        Self {
            repo,
            selected_index,
        }
    }

    /// Get the number of actions
    pub fn action_count(&self) -> usize {
        Action::all().len()
    }

    /// Get the currently selected action
    pub fn selected_action(&self) -> Option<Action> {
        if self.selected_index < self.action_count() {
            Some(Action::all()[self.selected_index].clone())
        } else {
            None
        }
    }

    /// Select next action
    pub fn select_next(&mut self) {
        let count = self.action_count();
        if count > 0 {
            self.selected_index = (self.selected_index + 1) % count;
        }
    }

    /// Select previous action
    pub fn select_previous(&mut self) {
        let count = self.action_count();
        if count > 0 {
            self.selected_index = if self.selected_index == 0 {
                count - 1
            } else {
                self.selected_index - 1
            };
        }
    }

    /// Set selected index
    pub fn select(&mut self, index: usize) {
        if index < self.action_count() {
            self.selected_index = index;
        }
    }

    /// Render the action menu
    pub fn render(&self, frame: &mut Frame, area: Rect) {
        // Clear the area behind the popup
        frame.render_widget(Clear, area);

        // Create menu items
        let items: Vec<ListItem> = Action::all()
            .iter()
            .enumerate()
            .map(|(i, action)| {
                let style = if i == self.selected_index {
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::Cyan)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                };

                let shortcut = action.shortcut();
                let desc = action.description();

                ListItem::new(Line::from(vec![
                    Span::styled(format!("[{}]", shortcut), Style::default().fg(Color::Cyan)),
                    Span::raw(" "),
                    Span::styled(desc, style),
                ]))
            })
            .collect();

        // Create the menu block
        let menu = List::new(items)
            .block(
                Block::default()
                    .title(format!(" Actions: {} ", self.repo.name))
                    .title_alignment(Alignment::Center)
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Cyan))
                    .style(Style::default().bg(Color::Black)),
            )
            .style(Style::default());

        frame.render_widget(menu, area);
    }
}

/// Calculate centered popup rectangle
pub fn centered_popup(width_percent: u16, height_percent: u16, area: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage((100 - height_percent) / 2),
                Constraint::Percentage(height_percent),
                Constraint::Percentage((100 - height_percent) / 2),
            ]
            .as_ref(),
        )
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage((100 - width_percent) / 2),
                Constraint::Percentage(width_percent),
                Constraint::Percentage((100 - width_percent) / 2),
            ]
            .as_ref(),
        )
        .split(popup_layout[1])[1]
}

/// Calculate centered help popup rectangle (fixed size)
pub fn centered_help_popup(area: Rect) -> Rect {
    // Fixed size for help panel
    let width = 60.min(area.width.saturating_sub(4));
    let height = 28.min(area.height.saturating_sub(4));

    let x = (area.width - width) / 2;
    let y = (area.height - height) / 2;

    Rect::new(x, y, width, height)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_action_menu_creation() {
        let repo = Repository::test_repo();
        let menu = ActionMenu::new(&repo, 0);
        assert_eq!(menu.action_count(), 7);
    }

    #[test]
    fn test_action_menu_selection() {
        let repo = Repository::test_repo();
        let mut menu = ActionMenu::new(&repo, 0);

        assert_eq!(menu.selected_action(), Some(Action::CdAndCloud));

        menu.select_next();
        assert_eq!(menu.selected_action(), Some(Action::OpenWebStorm));

        menu.select_previous();
        assert_eq!(menu.selected_action(), Some(Action::CdAndCloud));
    }

    #[test]
    fn test_action_menu_wrap() {
        let repo = Repository::test_repo();
        let mut menu = ActionMenu::new(&repo, 0);

        // Wrap around from start (now 7 actions, so index 6 is the last)
        menu.select_previous();
        assert_eq!(menu.selected_index, 6);

        // Wrap around from end
        menu.select_next();
        assert_eq!(menu.selected_index, 0);
    }

    #[test]
    fn test_centered_popup() {
        let area = Rect::new(0, 0, 100, 50);
        let popup = centered_popup(50, 40, area);

        assert_eq!(popup.width, 50);
        assert_eq!(popup.height, 20);
        assert_eq!(popup.x, 25);
        assert_eq!(popup.y, 15);
    }
}
