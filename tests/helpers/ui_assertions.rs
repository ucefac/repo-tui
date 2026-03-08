//! UI assertions for testing
//!
//! Provides assertions for verifying UI state in tests

use ratatui::buffer::Buffer;
use ratatui::style::Color;

/// Assert that a repository is displayed with the correct format
pub fn assert_repo_displayed(buffer: &Buffer, name: &str, scope: &str) {
    let expected = format!("@{}/{}", scope, name);
    let content = buffer_content(buffer);

    assert!(
        content.contains(&expected),
        "Expected to find repository display '{}' in buffer, but got:\n{}",
        expected,
        content
    );
}

/// Assert that a scope has a specific color
pub fn assert_scope_color(buffer: &Buffer, scope: &str, expected_color: Color) {
    // Find the position of the scope in the buffer
    let content = buffer_content(buffer);
    if let Some(pos) = content.find(scope) {
        let (x, y) = index_to_pos(buffer, pos);
        let cell = buffer.get(x, y);

        assert_eq!(
            cell.style().fg,
            Some(expected_color),
            "Expected scope '{}' to have color {:?}, but got {:?}",
            scope,
            expected_color,
            cell.style().fg
        );
    } else {
        panic!("Scope '{}' not found in buffer", scope);
    }
}

/// Assert that a path is shown in the status bar
pub fn assert_path_in_status_bar(buffer: &Buffer, path: &str) {
    let content = buffer_content(buffer);
    assert!(
        content.contains(path),
        "Expected path '{}' in status bar, but got:\n{}",
        path,
        content
    );
}

/// Assert that main directories are listed
pub fn assert_main_directories_listed(buffer: &Buffer, dirs: &[&str]) {
    let content = buffer_content(buffer);
    for dir in dirs {
        assert!(
            content.contains(dir),
            "Expected main directory '{}' to be listed, but got:\n{}",
            dir,
            content
        );
    }
}

/// Assert that a specific text appears in the buffer
pub fn assert_text_present(buffer: &Buffer, text: &str) {
    let content = buffer_content(buffer);
    assert!(
        content.contains(text),
        "Expected text '{}' to be present, but got:\n{}",
        text,
        content
    );
}

/// Assert that a specific text does NOT appear in the buffer
pub fn assert_text_not_present(buffer: &Buffer, text: &str) {
    let content = buffer_content(buffer);
    assert!(
        !content.contains(text),
        "Expected text '{}' to NOT be present, but it was found in:\n{}",
        text,
        content
    );
}

/// Assert that the buffer shows an empty state
pub fn assert_empty_state(buffer: &Buffer) {
    let content = buffer_content(buffer);
    let empty_indicators = ["no repositories", "empty", "no repos", "nothing found"];

    let has_indicator = empty_indicators
        .iter()
        .any(|&indicator| content.to_lowercase().contains(indicator));

    assert!(
        has_indicator,
        "Expected empty state indicator, but got:\n{}",
        content
    );
}

/// Assert that a specific style is applied to a cell
pub fn assert_cell_style(
    buffer: &Buffer,
    x: u16,
    y: u16,
    expected_fg: Option<Color>,
    expected_bg: Option<Color>,
) {
    let cell = buffer.get(x, y);
    let style = cell.style();

    if let Some(fg) = expected_fg {
        assert_eq!(
            style.fg,
            Some(fg),
            "Expected foreground color {:?} at ({}, {}), but got {:?}",
            fg,
            x,
            y,
            style.fg
        );
    }

    if let Some(bg) = expected_bg {
        assert_eq!(
            style.bg,
            Some(bg),
            "Expected background color {:?} at ({}, {}), but got {:?}",
            bg,
            x,
            y,
            style.bg
        );
    }
}

/// Get the full content of a buffer as a string
fn buffer_content(buffer: &Buffer) -> String {
    let mut lines = Vec::new();
    for y in 0..buffer.area().height {
        let mut line = String::new();
        for x in 0..buffer.area().width {
            let cell = buffer.get(x, y);
            line.push(cell.symbol().chars().next().unwrap_or(' '));
        }
        lines.push(line.trim_end().to_string());
    }
    lines.join("\n")
}

/// Convert a linear index to (x, y) position
fn index_to_pos(buffer: &Buffer, index: usize) -> (u16, u16) {
    let width = buffer.area().width as usize;
    let y = index / width;
    let x = index % width;
    (x as u16, y as u16)
}

/// Assert that the buffer contains a line starting with specific text
pub fn assert_line_starts_with(buffer: &Buffer, line_idx: u16, prefix: &str) {
    let mut line = String::new();
    for x in 0..buffer.area().width {
        let cell = buffer.get(x, line_idx);
        line.push(cell.symbol().chars().next().unwrap_or(' '));
    }

    assert!(
        line.trim().starts_with(prefix),
        "Expected line {} to start with '{}', but got: '{}'",
        line_idx,
        prefix,
        line.trim()
    );
}

/// Assert that the buffer shows a title
pub fn assert_title_present(buffer: &Buffer, title: &str) {
    let content = buffer_content(buffer);
    // Titles are often centered, so we check for the content without position constraints
    assert!(
        content.contains(title),
        "Expected title '{}' to be present in buffer:\n{}",
        title,
        content
    );
}

/// Assert that the buffer shows help text
pub fn assert_help_text_present(buffer: &Buffer, help_text: &str) {
    let content = buffer_content(buffer);
    assert!(
        content.contains(help_text),
        "Expected help text '{}' to be present, but got:\n{}",
        help_text,
        content
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::layout::Rect;

    fn create_test_buffer(content: &str) -> Buffer {
        let lines: Vec<&str> = content.lines().collect();
        let height = lines.len() as u16;
        let width = lines.iter().map(|l| l.len()).max().unwrap_or(0) as u16;

        let mut buffer = Buffer::empty(Rect::new(0, 0, width, height));

        for (y, line) in lines.iter().enumerate() {
            for (x, ch) in line.chars().enumerate() {
                buffer
                    .get_mut(x as u16, y as u16)
                    .set_symbol(&ch.to_string());
            }
        }

        buffer
    }

    #[test]
    fn test_assert_text_present() {
        let buffer = create_test_buffer("Hello World\nTest Line");
        assert_text_present(&buffer, "Hello");
        assert_text_present(&buffer, "Test Line");
    }

    #[test]
    #[should_panic]
    fn test_assert_text_present_fails() {
        let buffer = create_test_buffer("Hello World");
        assert_text_present(&buffer, "Not Present");
    }

    #[test]
    fn test_assert_text_not_present() {
        let buffer = create_test_buffer("Hello World");
        assert_text_not_present(&buffer, "Not Present");
    }

    #[test]
    fn test_buffer_content() {
        let buffer = create_test_buffer("Line1\nLine2");
        let content = buffer_content(&buffer);
        assert!(content.contains("Line1"));
        assert!(content.contains("Line2"));
    }
}
