# Phase 1 MVP UI Design Specification

## 1. Color System

### 1.1 Dark Theme (Default)

| Semantic Name | Color Value | Ratatui Code | Usage |
|---------------|-------------|--------------|-------|
| Primary | #58A6FF | `Color::Rgb(88, 166, 255)` | 强调元素、焦点边框、搜索框激活态 |
| Success | #3DB950 | `Color::Rgb(63, 185, 80)` | Clean状态、成功提示 |
| Warning | #D29922 | `Color::Rgb(210, 153, 34)` | 警告信息、Loading状态 |
| Error | #F85149 | `Color::Rgb(248, 81, 73)` | 错误提示、Dirty状态 |
| Selected BG | #388BFD | `Color::Rgb(56, 139, 253)` | 选中项背景 |
| Selected FG | #FFFFFF | `Color::White` | 选中项文字 |
| Border Focused | #00FFFF | `Color::Cyan` | 聚焦边框 |
| Border Normal | #808080 | `Color::DarkGray` | 普通边框 |
| Border Active | #58A6FF | `Color::Rgb(88, 166, 255)` | 激活态边框 |
| Text Primary | #FFFFFF | `Color::White` | 主要文字 |
| Text Secondary | #808080 | `Color::Gray` | 次要文字、统计信息 |
| Text Muted | #404040 | `Color::DarkGray` | Placeholder、禁用状态 |
| Cursor | #58A6FF | `Color::Rgb(88, 166, 255)` | 光标 |
| Highlight | #58A6FF | `Color::Rgb(88, 166, 255)` | 高亮 |
| Background | #000000 | `Color::Black` | 背景色 |

### 1.2 Light Theme

| Semantic Name | Color Value | Ratatui Code | Usage |
|---------------|-------------|--------------|-------|
| Primary | #0969DA | `Color::Rgb(9, 105, 218)` | 强调元素、焦点边框、搜索框激活态 |
| Success | #1A7F37 | `Color::Rgb(26, 127, 55)` | Clean状态、成功提示 |
| Warning | #9A6700 | `Color::Rgb(154, 103, 0)` | 警告信息、Loading状态 |
| Error | #D1242F | `Color::Rgb(209, 36, 47)` | 错误提示、Dirty状态 |
| Selected BG | #0969DA | `Color::Rgb(9, 105, 218)` | 选中项背景 |
| Selected FG | #FFFFFF | `Color::White` | 选中项文字 |
| Border Focused | #0000FF | `Color::Blue` | 聚焦边框 |
| Border Normal | #808080 | `Color::DarkGray` | 普通边框 |
| Border Active | #0969DA | `Color::Rgb(9, 105, 218)` | 激活态边框 |
| Text Primary | #000000 | `Color::Black` | 主要文字 |
| Text Secondary | #808080 | `Color::Gray` | 次要文字、统计信息 |
| Text Muted | #808080 | `Color::Gray` | Placeholder、禁用状态 |
| Cursor | #0969DA | `Color::Rgb(9, 105, 218)` | 光标 |
| Highlight | #0969DA | `Color::Rgb(9, 105, 218)` | 高亮 |
| Background | #FFFFFF | `Color::White` | 背景色 |

### 1.3 Status Colors

#### Git Status Indicators

| Status | Icon | Dark Theme Color | Light Theme Color |
|--------|------|------------------|-------------------|
| Dirty | `●` | `#F85149` (Red) | `#D1242F` (Red) |
| Clean | `✓` | `#3DB950` (Green) | `#1A7F37` (Green) |
| Unknown | `○` | `#808080` (Gray) | `#808080` (Gray) |

#### Selection States

| State | Style Definition |
|-------|------------------|
| Selected | `bg(Color::Rgb(56, 139, 253)) + fg(Color::White) + bold` |
| Unselected | `fg(Color::White)` (Dark) / `fg(Color::Black)` (Light) |
| Focused | `fg(Color::Cyan)` (Dark) / `fg(Color::Blue)` (Light) |

#### Component States

| Component | Normal State | Focused State |
|-----------|--------------|---------------|
| Search Box | Border: DarkGray | Border: Cyan (Dark) / Blue (Light) |
| List Item | Default text | Selected BG + Bold |
| Menu Item | Default text | Highlight BG |

## 2. Layout Specifications

### 2.1 Main Interface Layout

```
┌────────────────────────────────────────────────────────────────┐  ─┐
│ 🔍 Search: [react__________________________]            [15/342]│   │ 3 rows
├────────────────────────────────────────────────────────────────┤   │
│                                                                │   │
│ ╭─ Repositories (15/342) ────────────────────────────────────╮ │   │
│ │ ▌ github_facebook_react        main     ● dirty          │ │   │
│ │   web_react_native_docs        main     ✓ clean          │ │   │
│ │   personal_react_playground    feat-xy  ✓ clean          │ │   │ Min 5 rows
│ │                                                            │ │   │
│ │                                                            │ │   │
│ ╰────────────────────────────────────────────────────────────╯ │   │
│                                                                │   │
├────────────────────────────────────────────────────────────────┤   │
│ [j/k] Nav  [g/G] Jump  [/] Search  [Enter] Open  [?] Help  [q] │   │ 3 rows
└────────────────────────────────────────────────────────────────┘  ─┘
```

**Layout Constraints:**

```rust
use ratatui::layout::{Constraint, Direction, Layout};

fn create_main_layout(area: Rect) -> Vec<Rect> {
    Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),   // Search box (1 row content + 2 borders)
            Constraint::Min(5),      // Repository list (minimum 5 rows)
            Constraint::Length(3),   // Status bar (1 row content + 2 borders)
        ])
        .split(area)
}
```

**Component Dimensions:**

| Component | Min Height | Preferred Height | Notes |
|-----------|------------|------------------|-------|
| Search Box | 3 | 3 | Fixed height with borders |
| Repo List | 5 | Fill remaining | Scrollable, virtual list |
| Status Bar | 3 | 3 | Fixed height with borders |
| Total | 11 | Auto | Minimum terminal: 80x24 |

### 2.2 Directory Chooser Layout

**Modal Popup (80% width, 80% height):**

```
┌──────────────────────────────────────────────────────────────────────┐
│                    📁 Select Main Directory                           │  1 row
├──────────────────────────────────────────────────────────────────────┤
│ 📂 /home/username/projects                                           │  3 rows
├──────────────────────────────────────────────────────────────────────┤
│ 📊 12 subdirectories | 🗂️ 42 Git repositories                         │  2 rows
├──────────────────────────────────────────────────────────────────────┤
│ ╭─ Directories (12) ───────────────────────────────────────────────╮ │
│ │   ../                                                            │ │
│ │   Desktop/                                                       │ │
│ │ ▌ Documents/                                                     │ │
│ │   Downloads/                                                     │ │ Min 5 rows
│ │   Projects/                                                      │ │
│ │                                                                  │ │
│ ╰──────────────────────────────────────────────────────────────────╯ │
├──────────────────────────────────────────────────────────────────────┤
│ [j/k] Navigate  [Enter] Select  [←] Back  [q] Cancel                 │  1 row
└──────────────────────────────────────────────────────────────────────┘
```

**Layout Constraints:**

```rust
fn create_dir_chooser_layout(area: Rect) -> Vec<Rect> {
    Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),   // Title
            Constraint::Length(3),   // Current path
            Constraint::Length(2),   // Stats
            Constraint::Min(5),      // Directory list
            Constraint::Length(1),   // Spacer
            Constraint::Length(3),   // Help text
        ])
        .split(area)
}
```

### 2.3 Action Menu Layout

**Centered Popup (50% width, auto height):**

```
┌────────────────────────────────────────────┐
│ Actions: github_facebook_react             │
├────────────────────────────────────────────┤
│ [c] cd + cloud (claude)                    │
│ [w] Open in WebStorm                       │
│ [v] Open in VS Code                        │
│ [f] Open in Finder/Explorer                │
│ [q] Cancel                                 │
└────────────────────────────────────────────┘
```

**Dimensions:**
- Width: 50% of terminal width (min 40 cols, max 60 cols)
- Height: Auto based on action count + 2 (borders)
- Position: Centered

### 2.4 Help Panel Layout

**Centered Popup (60% width, 70% height):**

```
┌──────────────────────────────────────────────────────────────────┐
│ Keyboard Shortcuts                                               │
├──────────────────────────────────────────────────────────────────┤
│ Navigation                                                        │
│   j/↓     Move down                                             │
│   k/↑     Move up                                               │
│   g       Go to top                                             │
│   G       Go to bottom                                          │
│                                                                  │
│ Search                                                           │
│   /       Focus search                                          │
│   Esc     Clear search / Close panel                            │
│                                                                  │
│ Actions                                                          │
│   Enter   Open action menu                                      │
│   o       Open action menu                                      │
│   r       Refresh list                                          │
│   ?       Show this help                                        │
│   q       Quit                                                  │
└──────────────────────────────────────────────────────────────────┘
```

### 2.5 Minimum Terminal Requirements

**Minimum Supported Size:** 80 columns × 24 rows

**Size Validation:**

```rust
pub const MIN_TERMINAL_WIDTH: u16 = 80;
pub const MIN_TERMINAL_HEIGHT: u16 = 24;

fn check_terminal_size(area: Rect) -> bool {
    area.width >= MIN_TERMINAL_WIDTH && area.height >= MIN_TERMINAL_HEIGHT
}
```

**Size Warning Display:**

```
┌────────────────────────────────────────────────────────────────┐
│                                                                │
│                    Terminal too small!                         │
│                                                                │
│              Minimum size: 80x24                               │
│              Current size: 60x20                               │
│                                                                │
│            Please resize your terminal.                        │
│                                                                │
└────────────────────────────────────────────────────────────────┘
```

## 3. Component Styles

### 3.1 Search Box

**Structure:**
```
┌─ 🔍 Search (active) ───────────────────────────────────────────┐
│ react▌                                                        │
└────────────────────────────────────────────────────────────────┘
```

**Style Definitions:**

```rust
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, Paragraph};

impl SearchBox {
    fn render(&self, area: Rect, buf: &mut Buffer) {
        // Border style based on focus state
        let border_style = if self.focused {
            Style::default().fg(Color::Cyan)  // Dark theme focused
        } else {
            Style::default().fg(Color::DarkGray)
        };

        let block = Block::default()
            .title(if self.focused { "🔍 Search (active)" } else { "🔍 Search" })
            .borders(Borders::ALL)
            .border_style(border_style);

        // Text style
        let text_style = if self.query.is_empty() {
            Style::default().fg(Color::DarkGray)  // Placeholder style
        } else {
            Style::default().fg(Color::White)
        };

        // Cursor indicator
        let display_text = if self.focused && !self.query.is_empty() {
            format!("{}▌", self.query)  // Block cursor
        } else {
            self.query.to_string()
        };

        Paragraph::new(display_text)
            .block(block)
            .style(text_style)
            .render(area, buf);
    }
}
```

**Behavior:**
- **Normal State:** Gray border, placeholder text in muted color
- **Focused State:** Cyan/Blue border, title shows "(active)", cursor visible
- **Empty Query:** Shows placeholder "Type to search..." in muted color
- **With Query:** Shows query text + block cursor (▌) when focused

### 3.2 Repository List Item

**Structure:**
```
Wide (>100 cols):      Medium (60-100):       Narrow (<60):
▌ github_facebook_react    ▌ github_facebook_react    ▌ github_facebook_react
  main    ● dirty            main                       (no meta)
```

**Style Definitions:**

```rust
fn render_repo_item(
    repo: &Repository,
    is_selected: bool,
    show_git_status: bool,
    show_branch: bool,
    theme: &Theme,
) -> ListItem {
    let prefix = if is_selected { "▌ " } else { "  " };

    // Git status indicator
    let status_icon = if show_git_status {
        if repo.is_dirty {
            ("● ", Style::default().fg(theme.error))
        } else {
            ("✓ ", Style::default().fg(theme.success))
        }
    } else {
        ("", Style::default())
    };

    // Branch info
    let branch_text = if show_branch {
        repo.branch
            .as_ref()
            .map(|b| format!("{}", b))
            .unwrap_or_default()
    } else {
        String::new()
    };

    // Base style
    let mut style = Style::default().fg(theme.text_primary);
    if is_selected {
        style = style
            .bg(theme.selected_bg)
            .fg(Color::White)
            .add_modifier(Modifier::BOLD);
    }

    let content = format!("{}{}{}{}",
        prefix,
        status_icon.0,
        repo.name,
        if branch_text.is_empty() { "".to_string() } else { format!("  {}", branch_text) }
    );

    ListItem::new(content).style(style)
}
```

**Visual Indicators:**

| Element | Symbol | Color | Condition |
|---------|--------|-------|-----------|
| Selection Marker | `▌` | White | Selected |
| Empty Marker | `  ` | - | Unselected |
| Dirty Status | `●` | Error Red | `is_dirty == true` |
| Clean Status | `✓` | Success Green | `is_dirty == false` |
| Branch Name | text | Secondary | `show_branch == true` |

### 3.3 Status Bar

**Structure:**
```
┌─ Status ──────────────────────────────────────────────────────┐
│ [j/k] Nav  [g/G] Jump  [/] Search  [Enter] Open  [?] Help [q] │
└────────────────────────────────────────────────────────────────┘
```

**Style Definitions:**

```rust
fn render_status_bar(
    area: Rect,
    app: &App,
    theme: &Theme,
) {
    let status_text = match &app.state {
        AppState::Loading => format!(" ⏳ {}", app.loading_message),
        AppState::Error => format!(" ⚠️ {}", app.error_message),
        _ => " [j/k] Nav  [g/G] Jump  [/] Search  [Enter] Open  [?] Help  [q] Quit ".to_string(),
    };

    let style = Style::default()
        .fg(theme.text_secondary)
        .bg(Color::DarkGray);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.border_normal));

    Paragraph::new(status_text)
        .block(block)
        .style(style)
        .render(area, buf);
}
```

**State-based Display:**

| State | Display |
|-------|---------|
| Normal | 快捷键提示 |
| Loading | ⏳ + 加载消息 |
| Error | ⚠️ + 错误消息 |
| Success | ✅ + 成功消息 (3秒后消失) |

### 3.4 Action Menu

**Structure:**
```
┌─ Actions: github_facebook_react ───────────────────────────────┐
│ [c] cd + cloud (claude)                                        │
│ [w] Open in WebStorm                                           │
│ [v] Open in VS Code                                            │
│ [f] Open in Finder/Explorer                                    │
│ [q] Cancel                                                     │
└────────────────────────────────────────────────────────────────┘
```

**Style Definitions:**

```rust
fn render_action_menu(
    area: Rect,
    repo: &Repository,
    theme: &Theme,
) {
    let items: Vec<ListItem> = Action::all()
        .iter()
        .map(|action| {
            let content = format!("[{}] {}", action.shortcut(), action.description());
            ListItem::new(content)
                .style(Style::default().fg(theme.text_primary))
        })
        .collect();

    let block = Block::default()
        .title(format!("Actions: {}", repo.name))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.primary));

    let list = List::new(items).block(block);

    // Center the popup
    let popup_area = centered_rect(50, 30, area);
    Clear.render(popup_area, buf);
    list.render(popup_area, buf);
}
```

**Key Style Features:**
- Title: Repository name with primary color border
- Keys: Wrapped in `[]` brackets
- Background: Clear previous content with `Clear` widget
- Border: Primary color (Cyan/Blue)

### 3.5 Help Panel

**Structure:**
```
┌─ Keyboard Shortcuts ───────────────────────────────────────────┐
│ Navigation                                                      │
│   j/↓     Move down                                            │
│   k/↑     Move up                                              │
│   g       Go to top                                            │
│ ...                                                             │
└────────────────────────────────────────────────────────────────┘
```

**Style Definitions:**

```rust
fn render_help(area: Rect, theme: &Theme) {
    const HELP_TEXT: &str = r#"Navigation
  j/↓     Move down
  k/↑     Move up
  g       Go to top
  G       Go to bottom
  Ctrl+d  Scroll down half-page
  Ctrl+u  Scroll up half-page

Search
  /       Focus search
  Esc     Clear search / Close panel

Actions
  Enter   Open action menu
  o       Open action menu
  r       Refresh list
  ?       Show this help
  q       Quit"#;

    let block = Block::default()
        .title("Keyboard Shortcuts")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.primary));

    let paragraph = Paragraph::new(HELP_TEXT)
        .block(block)
        .style(Style::default().fg(theme.text_primary));

    let popup_area = centered_rect(60, 70, area);
    Clear.render(popup_area, buf);
    paragraph.render(popup_area, buf);
}
```

**Layout:**
- Centered: 60% width, 70% height
- Sections separated by blank lines
- Keys left-aligned, descriptions follow

## 4. Responsive Design Rules

### 4.1 Width-Based Adaptation

| Terminal Width | Strategy | Repository Display |
|----------------|----------|-------------------|
| < 60 columns | Compact mode | Name only, no meta |
| 60-100 columns | Standard mode | Name + Branch |
| > 100 columns | Full mode | Name + Branch + Status + Details |

### 4.2 Implementation

```rust
fn get_display_mode(width: u16) -> DisplayMode {
    match width {
        0..=59 => DisplayMode::Compact,
        60..=100 => DisplayMode::Standard,
        _ => DisplayMode::Full,
    }
}

fn render_repo_list(
    area: Rect,
    repos: &[Repository],
    theme: &Theme,
) {
    let mode = get_display_mode(area.width);

    let (show_branch, show_status, show_path) = match mode {
        DisplayMode::Compact => (false, false, false),
        DisplayMode::Standard => (true, false, false),
        DisplayMode::Full => (true, true, true),
    };

    // Render with appropriate flags
    let list = RepoList::new(repos, theme)
        .show_branch(show_branch)
        .show_git_status(show_status)
        .show_path(show_path);
}
```

### 4.3 Long Text Truncation

**Repository Name Truncation:**

```rust
fn truncate_repo_name(name: &str, max_width: usize) -> String {
    if name.len() <= max_width {
        name.to_string()
    } else if max_width > 6 {
        // Middle truncation: github_facebook_react -> github_...react
        let prefix_len = (max_width - 3) / 2;
        let suffix_len = max_width - 3 - prefix_len;
        format!("{}...{}",
            &name[..prefix_len],
            &name[name.len() - suffix_len..]
        )
    } else {
        // Simple truncation for very narrow widths
        format!("{}..", &name[..max_width - 2])
    }
}
```

**Path Display:**

```rust
fn format_path(path: &Path, max_width: usize) -> String {
    let path_str = path.display().to_string();

    if path_str.len() <= max_width {
        return path_str;
    }

    // Show last 2 components: /very/long/path/to/project -> .../to/project
    let components: Vec<_> = path.components().collect();
    if components.len() >= 2 {
        let last_two = components[components.len() - 2..]
            .iter()
            .map(|c| c.as_os_str().to_string_lossy())
            .collect::<Vec<_>>()
            .join("/");
        format!(".../{}", last_two)
    } else {
        format!("...{}", &path_str[path_str.len() - max_width + 3..])
    }
}
```

### 4.4 Height Adaptation

**Minimum Heights:**

| Component | Absolute Minimum | Recommended Minimum |
|-----------|------------------|---------------------|
| Terminal | 24 rows | 30+ rows |
| Repo List | 5 rows | 10+ rows |
| Popup | 10 rows | 15 rows |

**Scroll Behavior:**

```rust
impl RepoList {
    fn update_scroll(&mut self) {
        let visible_count = self.visible_count();
        let selected = self.selected_index.unwrap_or(0);

        // Scroll down
        if selected >= self.scroll_offset + visible_count {
            self.scroll_offset = selected.saturating_sub(visible_count - 1);
        }
        // Scroll up
        else if selected < self.scroll_offset {
            self.scroll_offset = selected;
        }
    }
}
```

## 5. Animation & Transitions

### 5.1 Loading States

**Spinner Animation (if implemented in Phase 2+):**

```rust
const SPINNER_FRAMES: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

fn render_loading(frame: &mut Frame, message: &str, tick: u64) {
    let spinner = SPINNER_FRAMES[(tick as usize) % SPINNER_FRAMES.len()];
    let text = format!("{} {}", spinner, message);
    // Render centered...
}
```

**Static Loading (Phase 1):**

```rust
fn render_loading(frame: &mut Frame, message: &str, theme: &Theme) {
    let text = format!("⏳ {}", message);
    let paragraph = Paragraph::new(text)
        .alignment(Alignment::Center)
        .style(Style::default().fg(theme.warning));
    frame.render_widget(paragraph, area);
}
```

### 5.2 State Transitions

**Modal Appearance:**
- No animation in Phase 1 (instant)
- Use `Clear` widget to erase background
- Border color: Primary
- Centered positioning

**Focus Changes:**
- Instant color change (no fade)
- Border: Gray → Cyan (Dark) / Blue (Light)
- Title suffix: `Search` → `Search (active)`

## 6. Typography

### 6.1 Text Styles

```rust
// Predefined styles
lazy_static! {
    static ref STYLES: Styles = Styles {
        normal: Style::default(),
        bold: Style::default().add_modifier(Modifier::BOLD),
        italic: Style::default().add_modifier(Modifier::ITALIC),
        underlined: Style::default().add_modifier(Modifier::UNDERLINED),
        dim: Style::default().add_modifier(Modifier::DIM),
    };
}
```

### 6.2 Icons & Symbols

| Purpose | Symbol | Unicode | Fallback |
|---------|--------|---------|----------|
| Search | 🔍 | U+1F50D | `>` |
| Directory | 📁 | U+1F4C1 | `[D]` |
| Folder (current) | 📂 | U+1F4C2 | `[C]` |
| Dirty | ● | U+25CF | `[M]` |
| Clean | ✓ | U+2713 | `[C]` |
| Loading | ⏳ | U+23F3 | `...` |
| Error | ⚠️ | U+26A0 | `!` |
| Success | ✅ | U+2705 | `+` |
| Branch |  | (text) | `@` |
| Selection | ▌ | U+258C | `>` |
| Cursor | ▌ | U+258C | `_` |

**Note:** Use ASCII fallbacks if Unicode support is limited.

## 7. Implementation Checklist

### 7.1 Theme System

- [x] Define color constants in `src/constants.rs`
- [x] Implement `Theme` struct in `src/ui/theme.rs`
- [x] Add dark theme colors
- [x] Add light theme colors
- [x] Implement style helper methods
- [x] Add theme selection from config

### 7.2 Layout System

- [x] Define minimum terminal size constants
- [x] Implement main layout (search + list + status)
- [x] Implement directory chooser layout
- [x] Implement popup centering helper
- [x] Add terminal size validation

### 7.3 Components

- [x] SearchBox widget with focus states
- [x] RepoList widget with virtual scrolling
- [x] DirChooser widget for directory selection
- [x] Action menu popup
- [x] Help panel popup
- [x] Status bar with state indicators
- [x] Size warning screen

### 7.4 Responsive Design

- [x] Display mode detection (<60, 60-100, >100)
- [x] Repository name truncation
- [x] Path truncation (last 2 components)
- [x] Meta data visibility control
- [ ] Detail panel for >100 cols (Phase 2)

### 7.5 Accessibility

- [x] High contrast selected state
- [x] Clear visual hierarchy
- [x] Consistent keyboard shortcuts
- [x] Status indicators with icons
- [ ] Screen reader support (Phase 3)

---

## Appendix: Ratatui Style Reference

### Color Constants

```rust
// Primary palette
pub const PRIMARY: Color = Color::Rgb(88, 166, 255);
pub const SUCCESS: Color = Color::Rgb(63, 185, 80);
pub const WARNING: Color = Color::Rgb(210, 153, 34);
pub const ERROR: Color = Color::Rgb(248, 81, 73);

// UI colors
pub const SELECTED_BG: Color = Color::Rgb(56, 139, 253);
pub const BORDER_FOCUSED: Color = Color::Cyan;
pub const BORDER_NORMAL: Color = Color::DarkGray;

// Text colors
pub const TEXT_PRIMARY: Color = Color::White;
pub const TEXT_SECONDARY: Color = Color::Gray;
pub const TEXT_MUTED: Color = Color::DarkGray;
```

### Style Builders

```rust
// Selected item
Style::default()
    .bg(Color::Rgb(56, 139, 253))
    .fg(Color::White)
    .add_modifier(Modifier::BOLD);

// Focused border
Style::default().fg(Color::Cyan);

// Git status - Dirty
Style::default().fg(Color::Rgb(248, 81, 73));

// Git status - Clean
Style::default().fg(Color::Rgb(63, 185, 80));

// Placeholder text
Style::default().fg(Color::DarkGray);
```

### Layout Helpers

```rust
// Centered popup
pub fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
```

---

**Document Version:** 1.0  
**Last Updated:** 2026-03-06  
**Status:** Phase 1 MVP Design Complete
