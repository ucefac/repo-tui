//! Mock terminal for UI testing
//!
//! Uses Ratatui's TestBackend to render UI without a real terminal.
//! This allows programmatic verification of rendered content.

use ratatui::backend::TestBackend;
use ratatui::buffer::Buffer;
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::Terminal;

/// Mock terminal for testing UI rendering
pub struct MockTerminal {
    terminal: Terminal<TestBackend>,
}

impl MockTerminal {
    /// Create a new mock terminal with specified dimensions
    pub fn new(width: u16, height: u16) -> Self {
        let backend = TestBackend::new(width, height);
        let terminal = Terminal::new(backend).unwrap();

        Self { terminal }
    }

    /// Get the terminal width
    pub fn width(&self) -> u16 {
        self.terminal.size().unwrap().width
    }

    /// Get the terminal height
    pub fn height(&self) -> u16 {
        self.terminal.size().unwrap().height
    }

    /// Get the current buffer content
    pub fn buffer(&self) -> &Buffer {
        self.terminal.backend().buffer()
    }

    /// Get a specific cell from the buffer
    pub fn get_cell(&self, x: u16, y: u16) -> Option<&ratatui::buffer::Cell> {
        self.buffer().cell((x, y))
    }

    /// Check if the buffer contains a string
    pub fn contains(&self, text: &str) -> bool {
        let buffer = self.buffer();
        let content = buffer
            .content
            .chunks(buffer.area.width as usize)
            .map(|row| {
                row.iter()
                    .map(|cell| cell.symbol())
                    .collect::<String>()
                    .trim_end()
                    .to_string()
            })
            .collect::<Vec<_>>()
            .join("\n");

        content.contains(text)
    }

    /// Check if a specific row contains a string
    pub fn row_contains(&self, row: u16, text: &str) -> bool {
        let buffer = self.buffer();
        let width = buffer.area.width as usize;
        let start = row as usize * width;
        let end = start + width;

        if end > buffer.content.len() {
            return false;
        }

        let row_content: String = buffer.content[start..end]
            .iter()
            .map(|cell| cell.symbol())
            .collect();

        row_content.contains(text)
    }

    /// Assert that the buffer contains a string (panics with context if not)
    pub fn assert_contains(&self, text: &str) {
        assert!(
            self.contains(text),
            "Expected buffer to contain '{}'. Buffer content:\n{}",
            text,
            self.to_string()
        );
    }

    /// Assert that the buffer does NOT contain a string
    pub fn assert_not_contains(&self, text: &str) {
        assert!(
            !self.contains(text),
            "Expected buffer NOT to contain '{}'. Buffer content:\n{}",
            text,
            self.to_string()
        );
    }

    /// Get the entire buffer as a string (for debugging)
    pub fn to_string(&self) -> String {
        let buffer = self.buffer();
        buffer
            .content
            .chunks(buffer.area.width as usize)
            .map(|row| {
                row.iter()
                    .map(|cell| cell.symbol())
                    .collect::<String>()
                    .trim_end()
                    .to_string()
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Print the buffer (useful for debugging)
    pub fn print(&self) {
        println!("\n{}", self.to_string());
    }

    /// Resize the terminal
    pub fn resize(&mut self, width: u16, height: u16) {
        // TestBackend doesn't properly support resize, so we recreate it
        let backend = TestBackend::new(width, height);
        self.terminal = Terminal::new(backend).unwrap();
    }

    /// Draw a widget to the terminal
    pub fn draw<F>(&mut self, f: F)
    where
        F: FnOnce(&mut ratatui::Frame),
    {
        self.terminal.draw(f).unwrap();
    }

    /// Clear the terminal
    pub fn clear(&mut self) {
        self.terminal.clear().unwrap();
    }
}

impl Default for MockTerminal {
    fn default() -> Self {
        Self::new(80, 24)
    }
}

/// Helper to create a KeyEvent
pub fn key_event(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

/// Helper to create a char KeyEvent
pub fn char_key(c: char) -> KeyEvent {
    KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE)
}

/// Helper to create a Ctrl+char KeyEvent
pub fn ctrl_key(c: char) -> KeyEvent {
    KeyEvent::new(KeyCode::Char(c), KeyModifiers::CONTROL)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::widgets::{Block, Borders, Paragraph};

    #[test]
    fn test_mock_terminal_new() {
        let term = MockTerminal::new(80, 24);
        assert_eq!(term.width(), 80);
        assert_eq!(term.height(), 24);
    }

    #[test]
    fn test_contains() {
        let mut term = MockTerminal::new(40, 10);

        term.draw(|f| {
            let text = Paragraph::new("Hello, World!");
            f.render_widget(text, f.area());
        });

        assert!(term.contains("Hello"));
        assert!(term.contains("World"));
        assert!(!term.contains("Goodbye"));
    }

    #[test]
    fn test_row_contains() {
        let mut term = MockTerminal::new(40, 10);

        term.draw(|f| {
            let block = Block::default().title("Test Title").borders(Borders::ALL);
            f.render_widget(block, f.area());
        });

        // Title should be on row 0
        assert!(term.row_contains(0, "Test Title"));
    }

    #[test]
    fn test_assert_contains() {
        let mut term = MockTerminal::new(40, 10);

        term.draw(|f| {
            let text = Paragraph::new("Test Content");
            f.render_widget(text, f.area());
        });

        term.assert_contains("Test Content");
    }

    #[test]
    fn test_resize() {
        let mut term = MockTerminal::new(80, 24);

        // Resize and clear to force backend to update
        term.resize(100, 30);
        term.clear();

        // Force a redraw to update the buffer
        term.draw(|f| {
            let block = Block::default();
            f.render_widget(block, f.area());
        });

        assert_eq!(term.width(), 100);
        assert_eq!(term.height(), 30);
    }

    #[test]
    fn test_key_helpers() {
        let key = key_event(KeyCode::Enter);
        assert_eq!(key.code, KeyCode::Enter);
        assert_eq!(key.modifiers, KeyModifiers::NONE);

        let key = char_key('a');
        assert_eq!(key.code, KeyCode::Char('a'));

        let key = ctrl_key('c');
        assert_eq!(key.code, KeyCode::Char('c'));
        assert!(key.modifiers.contains(KeyModifiers::CONTROL));
    }
}
