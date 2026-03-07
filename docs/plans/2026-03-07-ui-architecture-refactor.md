# UI 架构重构计划 - Phase 2

**创建日期**: 2026-03-07  
**状态**: 待执行  
**优先级**: 中  
**预计工作量**: 5-7 小时

---

## 📋 概述

Phase 2 旨在解决 Phase 1 快速修复后遗留的架构问题，实现更清晰、更统一的 UI 组件设计。

### Phase 1 已完成
- ✅ 工具栏添加自动换行
- ✅ 缩小工具栏按键间距
- ✅ 搜索框添加导航支持
- ✅ Esc 行为修改（保留查询）

### Phase 2 目标
1. 创建统一的 `StatusBar` 组件
2. 重构搜索为"非模态聚焦状态"
3. 统一边距和对齐方式
4. 提升代码质量和可维护性

---

## 🎯 重构任务

### 任务 1: 创建统一 StatusBar 组件

**问题**: 状态栏和路径栏当前分开渲染，边距不一致，视觉上不统一。

**解决方案**: 创建统一的 `StatusBar` 组件，整合状态信息和路径显示。

**涉及文件**:
- `src/ui/widgets/status_bar.rs` (新建)
- `src/ui/widgets/mod.rs` (修改)
- `src/ui/render.rs` (修改)

**实现细节**:

```rust
// src/ui/widgets/status_bar.rs
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use std::path::Path;

use crate::ui::theme::Theme;

/// Unified status bar widget
pub struct StatusBar<'a> {
    pub status_message: &'a str,
    pub path: Option<&'a Path>,
    pub repo_count: Option<usize>,
    pub theme: &'a Theme,
    pub loading: bool,
    pub error: bool,
}

impl<'a> StatusBar<'a> {
    pub fn new(status_message: &'a str, theme: &'a Theme) -> Self {
        Self {
            status_message,
            path: None,
            repo_count: None,
            theme,
            loading: false,
            error: false,
        }
    }

    pub fn path(mut self, path: &'a Path) -> Self {
        self.path = Some(path);
        self
    }

    pub fn repo_count(mut self, count: usize) -> Self {
        self.repo_count = Some(count);
        self
    }

    pub fn loading(mut self, loading: bool) -> Self {
        self.loading = loading;
        self
    }

    pub fn error(mut self, error: bool) -> Self {
        self.error = error;
        self
    }
}

impl<'a> Widget for StatusBar<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // 统一使用 1px padding
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.theme.border_normal))
            .style(Style::default().bg(Color::DarkGray));

        // 状态文本（支持自动换行）
        let status_text = if self.loading {
            format!("⏳ {}", self.status_message)
        } else if self.error {
            format!("⚠️ {}", self.status_message)
        } else {
            self.status_message.to_string()
        };

        let status_paragraph = Paragraph::new(status_text)
            .block(block.clone())
            .style(Style::default().fg(self.theme.text_secondary))
            .wrap(Wrap { trim: true })
            .alignment(Alignment::Left);

        status_paragraph.render(area, buf);

        // 路径信息渲染在底部
        if let Some(path) = self.path {
            let path_text = format!(
                "📂 {}{}",
                path.display(),
                self.repo_count.map(|c| format!(" ({} repos)", c)).unwrap_or_default()
            );

            let path_paragraph = Paragraph::new(path_text)
                .style(Style::default().fg(self.theme.text_secondary))
                .alignment(Alignment::Left);

            // 渲染在底部区域
            let path_area = Rect::new(
                area.x,
                area.y + area.height - 1,
                area.width,
                1,
            );
            path_paragraph.render(path_area, buf);
        }
    }
}
```

**修改 `render.rs`**:

```rust
// src/ui/render.rs:144-189 替换为
fn render_status_bar_with_path(frame: &mut Frame, app: &mut App, area: Rect, theme: &Theme) {
    use crate::ui::widgets::StatusBar;

    let status_text = if app.loading {
        app.loading_message.as_deref().unwrap_or("Loading...")
    } else if let Some(ref error) = app.error_message {
        error.as_str()
    } else {
        "[j/k]Nav [g/G]Jump [/]Search [Enter]Open [r]Refresh [?]Help [q]Quit [m]ChangeDir"
    };

    let mut status_bar = StatusBar::new(status_text, theme)
        .loading(app.loading)
        .error(app.error_message.is_some());

    if let Some(ref main_dir) = app.main_dir {
        status_bar = status_bar.path(main_dir).repo_count(app.repositories.len());
    }

    // 保存点击区域
    app.path_bar_area = Some(area);
    
    frame.render_widget(status_bar, area);
}
```

**布局调整**:

```rust
// src/ui/render.rs:118-125
let chunks = Layout::default()
    .direction(Direction::Vertical)
    .constraints([
        Constraint::Length(3), // Search box
        Constraint::Min(5),    // Repository list
        Constraint::Length(2), // Status bar (从 4 行减少到 2 行)
    ])
    .split(area);
```

**工作量**: 2 小时  
**风险**: 中 - 需要验证最小终端高度

---

### 任务 2: 重构搜索为非模态状态

**问题**: 当前 `AppState::Searching` 是模态状态，搜索时无法导航，与 Elm 架构原则不符。

**解决方案**: 移除独立的 `Searching` 状态，改为 `search_active: bool` 标志。

**涉及文件**:
- `src/app/state.rs` (修改)
- `src/app/model.rs` (修改)
- `src/app/msg.rs` (修改)
- `src/app/update.rs` (修改)
- `src/handler/keyboard.rs` (修改)
- `src/ui/render.rs` (修改)

**核心设计原则**:
1. **激活方式**：仅通过 `/` 键激活搜索框
2. **聚焦行为**：搜索框聚焦时，所有字母键只用于输入，不触发任何功能
3. **导航支持**：通过方向键（↑↓Home/End）支持列表导航
4. **退出行为**：`Esc`/`Enter` 退出聚焦但保留查询，`Backspace` 删空时自动退出

**实现细节**:

#### Step 1: 修改 AppState

```rust
// src/app/state.rs
pub enum AppState {
    Running,
    ChoosingDir {
        path: PathBuf,
        entries: Vec<String>,
        selected_index: usize,
        scroll_offset: usize,
    },
    ShowingActions { repo: Repository },
    ShowingHelp,
    Loading { message: String },
    Error { message: String },
    Quit,
    // ❌ 移除：Searching
}
```

#### Step 2: 添加 search_active 字段

```rust
// src/app/model.rs
pub struct App {
    // ... 现有字段 ...
    pub search_active: bool,  // 新增：搜索框是否聚焦
}

impl App {
    pub fn new(msg_tx: mpsc::Sender<AppMsg>) -> Self {
        Self {
            // ... 现有初始化 ...
            search_active: false,
        }
    }
}
```

#### Step 3: 重构键盘处理

```rust
// src/handler/keyboard.rs:18-40
pub fn handle_key_event(key: KeyEvent, app: &mut App, runtime: &Runtime) {
    // Ctrl+C 全局处理
    if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
        let _ = app.msg_tx.try_send(AppMsg::Quit);
        return;
    }

    // 根据状态处理
    match &app.state {
        AppState::ShowingActions { .. } => handle_action_menu_keys(key, app, runtime),
        AppState::ShowingHelp => handle_help_keys(key, app),
        AppState::ChoosingDir { .. } => handle_chooser_keys(key, app, runtime),
        
        // 新增：Running 状态下检查搜索聚焦
        AppState::Running | AppState::Loading { .. } | AppState::Error { .. } => {
            if app.search_active {
                handle_search_input(key, app, runtime);
            } else {
                handle_running_keys(key, app, runtime);
            }
        }
        
        AppState::Quit => {}
    }
}

// 新增函数：处理搜索输入（修订版）
// 核心原则：搜索框聚焦时，只有方向键导航，所有字母键只输入
fn handle_search_input(key: KeyEvent, app: &mut App, _runtime: &Runtime) {
    match key.code {
        // === 退出/确认 ===
        KeyCode::Esc => {
            // 仅退出聚焦，保留查询
            app.search_active = false;
        }
        KeyCode::Enter => {
            // 确认搜索，退出聚焦
            app.search_active = false;
        }
        
        // === 编辑键 ===
        KeyCode::Backspace => {
            let _ = app.msg_tx.try_send(AppMsg::SearchBackspace);
        }
        
        // === 导航键（方向键，不影响搜索框内容）===
        KeyCode::Up => {
            let _ = app.msg_tx.try_send(AppMsg::PreviousRepo);
        }
        KeyCode::Down => {
            let _ = app.msg_tx.try_send(AppMsg::NextRepo);
        }
        KeyCode::Home => {
            let _ = app.msg_tx.try_send(AppMsg::JumpToTop);
        }
        KeyCode::End => {
            let _ = app.msg_tx.try_send(AppMsg::JumpToBottom);
        }
        
        // === 所有其他字符：只作为输入 ===
        // 包括 j/k/g/G/m/r/q/? 等功能键，搜索时全部屏蔽
        KeyCode::Char(c) => {
            let _ = app.msg_tx.try_send(AppMsg::SearchInput(c));
        }
        
        _ => {}
    }
}

// 修改 handle_running_keys
fn handle_running_keys(key: KeyEvent, app: &mut App, _runtime: &Runtime) {
    match key.code {
        // 激活搜索（唯一入口）
        KeyCode::Char('/') => {
            app.search_active = true;
        }
        
        // 其他功能键（正常状态下的快捷键）
        KeyCode::Char('j') | KeyCode::Down => {
            let _ = app.msg_tx.try_send(AppMsg::NextRepo);
        }
        KeyCode::Char('k') | KeyCode::Up => {
            let _ = app.msg_tx.try_send(AppMsg::PreviousRepo);
        }
        KeyCode::Char('g') if key.modifiers.contains(KeyModifiers::NONE) => {
            let _ = app.msg_tx.try_send(AppMsg::JumpToTop);
        }
        KeyCode::Char('G') => {
            let _ = app.msg_tx.try_send(AppMsg::JumpToBottom);
        }
        KeyCode::Char('r') => {
            let _ = app.msg_tx.try_send(AppMsg::Refresh);
        }
        KeyCode::Char('m') => {
            let _ = app.msg_tx.try_send(AppMsg::ChangeDirectory);
        }
        KeyCode::Char('q') => {
            let _ = app.msg_tx.try_send(AppMsg::Quit);
        }
        KeyCode::Char('?') => {
            let _ = app.msg_tx.try_send(AppMsg::ToggleHelp);
        }
        KeyCode::Enter => {
            let _ = app.msg_tx.try_send(AppMsg::OpenSelected);
        }
        _ => {}
    }
}
```

#### Step 4: 更新消息处理

```rust
// src/app/update.rs
AppMsg::SearchInput(c) => {
    // 不再检查 AppState::Searching
    app.search_query.push(c);
    app.search_active = true;  // 确保聚焦
    app.pending_search = Some(app.search_query.clone());
    
    runtime.dispatch_after(
        crate::app::msg::AppMsg::Tick,
        std::time::Duration::from_millis(constants::SEARCH_DEBOUNCE_MS),
    );
}

AppMsg::SearchBackspace => {
    app.search_query.pop();
    
    if app.search_query.is_empty() {
        app.search_active = false;  // 空查询时退出聚焦
        app.pending_search = None;
        app.apply_filter();
    } else {
        app.pending_search = Some(app.search_query.clone());
        runtime.dispatch_after(/* ... */);
    }
}
```

#### Step 5: 更新 UI 渲染

```rust
// src/ui/render.rs
let is_search_focused = app.search_active;  // 替代原来的 matches!(app.state, AppState::Searching)
```

**工作量**: 3-4 小时  
**风险**: 中 - 涉及多个文件，需要全面测试

---

## 🧪 测试策略

### 单元测试

**StatusBar 组件**:
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::backend::TestBackend;

    #[test]
    fn test_status_bar_render() {
        let backend = TestBackend::new(80, 5);
        let mut terminal = Terminal::new(backend).unwrap();
        let theme = Theme::dark();

        terminal.draw(|f| {
            let status_bar = StatusBar::new("Test", &theme)
                .path(Path::new("/tmp"))
                .repo_count(5);
            f.render_widget(status_bar, f.area());
        }).unwrap();
    }

    #[test]
    fn test_status_bar_wrap() {
        // 测试长文本自动换行
    }
}
```

**搜索行为测试**:
```rust
#[test]
fn test_search_input_does_not_trigger_m() {
    // 1. 按 / 激活搜索
    // 2. 按 m
    // 3. 验证：search_query == "m", state != ChoosingDir
}

#[test]
fn test_search_navigation_with_arrow_keys() {
    // 1. 按 / 激活搜索
    // 2. 输入 "test"
    // 3. 按 ↓
    // 4. 验证：selected_index + 1, search_query == "test"
}

#[test]
fn test_search_esc_retains_query() {
    // 1. 按 / 激活
    // 2. 输入 "hello"
    // 3. 按 Esc
    // 4. 验证：search_active = false, search_query == "hello"
}

#[test]
fn test_search_backspace_clears() {
    // 1. 按 / 激活
    // 2. 输入 "a"
    // 3. 按 Backspace
    // 4. 验证：search_active = false, search_query == "", 列表恢复全部
}
```

### 集成测试

```rust
// tests/search_navigation.rs
#[tokio::test]
async fn test_search_workflow() {
    // 完整流程：激活 → 输入 → 导航 → 退出 → 重新激活
    // 验证搜索框聚焦状态与列表导航的独立性
}

#[tokio::test]
async fn test_search_only_activates_with_slash() {
    // 验证只有 / 键能激活搜索框
    // 按其他字母键（如 j/k/m）不会激活搜索
}
```

---

## 📊 验收标准

### 功能验收
- [ ] 状态栏和路径栏视觉上统一对齐（上下两行，共用边框）
- [ ] 搜索只能通过 `/` 键激活
- [ ] 搜索框聚焦时，所有字母键（包括 j/k/g/m/r/q 等）只用于输入
- [ ] 搜索框聚焦时，方向键（↑↓Home/End）可以导航列表
- [ ] Esc 仅退出搜索框，保留查询
- [ ] Enter 确认搜索，退出搜索框，保留查询
- [ ] Backspace 删空时自动退出搜索框，恢复全部列表
- [ ] 窄终端上工具栏自动换行

### 质量验收
- [ ] 所有现有测试通过
- [ ] 新增测试覆盖核心功能（搜索激活、输入、导航、退出）
- [ ] Clippy 无警告
- [ ] 代码格式化

### 文档验收
- [ ] 更新 `docs/PATH_DISPLAY_FEATURE.md`
- [ ] 更新帮助面板说明（搜索行为说明）
- [ ] 添加 StatusBar 组件文档注释

---

## ⚠️ 风险评估

| 风险 | 影响 | 概率 | 缓解措施 |
|------|------|------|----------|
| 状态重构破坏现有逻辑 | 高 | 中 | 全面测试 + 逐步迁移 |
| 最小终端高度不足 | 中 | 低 | 调整约束 + 验证 |
| 搜索行为变更用户不习惯 | 低 | 低 | 文档说明 + 帮助更新 |

---

## 📅 实施计划

### 阶段 1: StatusBar 组件 (2 小时)
1. 创建 `src/ui/widgets/status_bar.rs`
2. 更新 `src/ui/widgets/mod.rs` 导出新组件
3. 更新 `src/ui/render.rs` 调用新组件
4. 添加单元测试
5. 验证布局（最小终端高度检查）

### 阶段 2: 搜索重构 (3-4 小时)
1. 修改 `src/app/state.rs` 移除 `Searching` 变体
2. 添加 `search_active: bool` 字段到 `src/app/model.rs`
3. 重构 `src/handler/keyboard.rs`：
   - 实现 `handle_search_input` 函数（方向键导航，字母键输入）
   - 更新 `handle_running_keys`（仅 `/` 激活搜索）
4. 更新 `src/app/update.rs` 消息处理
5. 更新 `src/ui/render.rs` 渲染逻辑
6. 全面测试（单元测试 + 集成测试）

### 阶段 3: 文档和清理 (1 小时)
1. 更新本文档
2. 更新帮助面板说明
3. 代码审查（Code Review）
4. 运行 `cargo clippy` 和 `cargo fmt`

---

## 🔗 相关文档

- [Phase 1 完成报告](./2026-03-07-ui-phase1-complete.md)
- [初始设计文档](./2026-03-07-main-directory-path-display-design.md)
- [开发指南](./DEVELOPMENT_GUIDE.md)

---

**最后更新**: 2026-03-07  
**维护者**: repotui Team

---

## 📝 修订历史

### 2026-03-07 - 搜索架构修订

**修订内容**:
- 明确搜索框激活方式：仅通过 `/` 键
- 修改搜索框聚焦行为：所有字母键只用于输入，不触发任何功能
- 导航方式：使用方向键（↑↓Home/End），而非 j/k 字母键
- 更新键盘处理逻辑和测试策略

**修订原因**:
- 避免搜索时输入 "main" 触发 `m` 键的切换目录功能
- 避免搜索时输入 "japan" 触发 `j` 键的导航功能
- 提供更直观的搜索体验（聚焦时字母=输入，方向键=导航）
