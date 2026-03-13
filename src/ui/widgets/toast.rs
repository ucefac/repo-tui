//! Toast notification widget

use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph, Widget},
};

/// Toast type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToastType {
    Success,
    Error,
    Warning,
    Info,
}

/// Toast notification
#[derive(Debug, Clone)]
pub struct Toast {
    /// Toast type
    pub toast_type: ToastType,
    /// Message content
    pub message: String,
    /// Display duration in milliseconds
    pub duration_ms: u64,
    /// Remaining time (for animation)
    pub remaining_ms: u64,
}

impl Toast {
    /// Create a success toast
    pub fn success(message: impl Into<String>) -> Self {
        Self {
            toast_type: ToastType::Success,
            message: message.into(),
            duration_ms: 3000,
            remaining_ms: 3000,
        }
    }

    /// Create an error toast
    pub fn error(message: impl Into<String>) -> Self {
        Self {
            toast_type: ToastType::Error,
            message: message.into(),
            duration_ms: 5000,
            remaining_ms: 5000,
        }
    }

    /// Create a warning toast
    pub fn warning(message: impl Into<String>) -> Self {
        Self {
            toast_type: ToastType::Warning,
            message: message.into(),
            duration_ms: 5000,
            remaining_ms: 5000,
        }
    }

    /// Create an info toast
    pub fn info(message: impl Into<String>) -> Self {
        Self {
            toast_type: ToastType::Info,
            message: message.into(),
            duration_ms: 2000,
            remaining_ms: 2000,
        }
    }

    /// Get toast colors based on type
    fn get_colors(&self) -> (Color, Color, &'static str) {
        match self.toast_type {
            ToastType::Success => (
                Color::Rgb(34, 197, 94),
                Color::Rgb(20, 83, 45),
                "✓",
            ),
            ToastType::Error => (
                Color::Rgb(239, 68, 68),
                Color::Rgb(127, 29, 29),
                "✗",
            ),
            ToastType::Warning => (
                Color::Rgb(245, 158, 11),
                Color::Rgb(120, 53, 15),
                "⚠",
            ),
            ToastType::Info => (
                Color::Rgb(59, 130, 246),
                Color::Rgb(30, 58, 138),
                "ℹ",
            ),
        }
    }
}

/// Default implementation
impl Default for Toast {
    fn default() -> Self {
        Self::info("")
    }
}

/// Calculate toast rectangle
///
/// Position: Bottom center, 2 rows from bottom
pub fn calculate_toast_rect(area: Rect, toast: &Toast) -> Rect {
    let message_width = toast.message.len() + 6; // icon + padding
    let toast_width = message_width.min(80).max(40) as u16;
    let toast_height = 3u16;

    let width = area.width.min(toast_width);
    let height = toast_height;

    // Center horizontally, 2 rows from bottom
    let x = area.x + (area.width.saturating_sub(width)) / 2;
    let y = area.y + area.height.saturating_sub(height + 2);

    Rect::new(x, y, width, height)
}

/// Toast widget renderer
pub struct ToastWidget<'a> {
    toast: &'a Toast,
}

impl<'a> ToastWidget<'a> {
    pub fn new(toast: &'a Toast) -> Self {
        Self { toast }
    }
}

impl Widget for ToastWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let (border_color, bg_color, icon) = self.toast.get_colors();

        // Create the content with icon prefix
        let content = format!(" {} {}", icon, self.toast.message);

        // Create block with border
        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Plain)
            .border_style(Style::default().fg(border_color))
            .style(Style::default().bg(bg_color));

        // Create paragraph
        let paragraph = Paragraph::new(Line::from(vec![Span::styled(
            content,
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )]))
        .block(block.clone())
        .alignment(Alignment::Center);

        // Render with background
        let inner_area = block.inner(area);
        block.render(area, buf);
        paragraph.render(inner_area, buf);
    }
}

/// Toast manager - handles multiple toasts
#[derive(Debug, Default)]
pub struct ToastManager {
    /// Current toasts
    pub toasts: Vec<Toast>,
}

impl ToastManager {
    /// Create a new toast manager
    pub fn new() -> Self {
        Self { toasts: Vec::new() }
    }

    /// Add a toast
    pub fn add(&mut self, toast: Toast) {
        self.toasts.push(toast);
    }

    /// Add success toast
    pub fn success(&mut self, message: impl Into<String>) {
        self.add(Toast::success(message));
    }

    /// Add error toast
    pub fn error(&mut self, message: impl Into<String>) {
        self.add(Toast::error(message));
    }

    /// Add warning toast
    pub fn warning(&mut self, message: impl Into<String>) {
        self.add(Toast::warning(message));
    }

    /// Add info toast
    pub fn info(&mut self, message: impl Into<String>) {
        self.add(Toast::info(message));
    }

    /// Update toasts (decrement timers)
    ///
    /// Call this every frame with delta time in milliseconds
    pub fn update(&mut self, delta_ms: u64) {
        self.toasts.retain_mut(|toast| {
            toast.remaining_ms = toast.remaining_ms.saturating_sub(delta_ms);
            toast.remaining_ms > 0
        });
    }

    /// Clear all toasts
    pub fn clear(&mut self) {
        self.toasts.clear();
    }

    /// Get current toast (most recent)
    pub fn current(&self) -> Option<&Toast> {
        self.toasts.last()
    }

    /// Check if there are any toasts
    pub fn has_toasts(&self) -> bool {
        !self.toasts.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_toast_success() {
        let toast = Toast::success("Test message");
        assert_eq!(toast.toast_type, ToastType::Success);
        assert_eq!(toast.message, "Test message");
        assert_eq!(toast.duration_ms, 3000);
    }

    #[test]
    fn test_toast_error() {
        let toast = Toast::error("Error message");
        assert_eq!(toast.toast_type, ToastType::Error);
        assert_eq!(toast.duration_ms, 5000);
    }

    #[test]
    fn test_toast_manager() {
        let mut manager = ToastManager::new();

        manager.success("Success!");
        manager.error("Error!");

        assert_eq!(manager.toasts.len(), 2);
        assert_eq!(manager.current().unwrap().toast_type, ToastType::Error);
    }

    #[test]
    fn test_toast_manager_update() {
        let mut manager = ToastManager::new();
        manager.info("Test");

        manager.update(1000);
        assert_eq!(manager.toasts.len(), 1);

        manager.update(2000);
        assert_eq!(manager.toasts.len(), 0);
    }
}
