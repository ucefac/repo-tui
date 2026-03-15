# 鼠标滚轮滚动支持计划

**日期**: 2026-03-15  
**任务**: 实现所有列表的鼠标滚轮逐行滚动支持  
**优先级**: 高

---

## 问题描述

鼠标滚动列表时，当前一次滚动 4 行，用户希望一次滚动 1 行。

## 根因分析

`src/handler/mouse.rs` 只处理鼠标点击事件（`MouseEventKind::Down`），未实现滚轮事件（`MouseEventKind::ScrollUp`/`ScrollDown`）处理。终端或 ratatui 的默认滚轮行为导致一次滚动 4 行。

## 解决方案

在 `src/handler/mouse.rs` 中添加滚轮事件处理，根据不同应用状态发送对应的导航消息，实现逐行滚动（每次 1 行）。

---

## 涉及列表

| # | 列表名称 | AppState | 滚动控制 | 导航消息 | Widget |
|---|----------|----------|----------|----------|--------|
| 1 | 主仓库列表 | `Running` | `app.scroll_offset` | `PreviousRepo`/`NextRepo` | `RepoList` |
| 2 | 目录选择器 | `ChoosingDir` | `scroll_offset` | `DirectoryNavUp`/`DirectoryNavDown` | `DirectoryChooser` |
| 3 | 主题选择器 | `SelectingTheme` | `scroll_offset` | `ThemeNavUp`/`ThemeNavDown` | `ThemeSelector` |
| 4 | 主目录管理器 | `ManagingDirs` | `scroll_offset` | `MainDirNavUp`/`MainDirNavDown` | `MainDirManager` |
| 5 | 帮助面板 | `ShowingHelp` | `scroll_offset` | 直接修改 offset | `HelpPanel` |
| 6 | 移动目标选择 | `SelectingMoveTarget` | `ListState` | 直接修改 `ListState` | - |
| 7 | 克隆目录选择 | `Cloning` | `ListState` | `ClonePreviousMainDir`/`CloneNextMainDir` | - |

---

## 任务分解

### Task 1: 修改 `src/handler/mouse.rs`

**目标**: 添加滚轮事件处理逻辑

**修改内容**:
```rust
use crossterm::event::{MouseButton, MouseEvent, MouseEventKind};

pub fn handle_mouse_event(event: MouseEvent, app: &App) -> Option<AppMsg> {
    match event.kind {
        // 现有点击处理
        MouseEventKind::Down(MouseButton::Left) => {
            // ... 现有代码
        }
        
        // 新增：滚轮向上滚动
        MouseEventKind::ScrollUp => {
            return Some(get_scroll_up_message(app));
        }
        
        // 新增：滚轮向下滚动
        MouseEventKind::ScrollDown => {
            return Some(get_scroll_down_message(app));
        }
        
        _ => {}
    }
    None
}

// 根据当前状态返回对应的向上导航消息
fn get_scroll_up_message(app: &App) -> AppMsg {
    match &app.state {
        AppState::Running => AppMsg::PreviousRepo,
        AppState::ChoosingDir { .. } => AppMsg::DirectoryNavUp,
        AppState::SelectingTheme { .. } => AppMsg::ThemeNavUp,
        AppState::ManagingDirs { .. } => AppMsg::MainDirNavUp,
        AppState::ShowingHelp { .. } => AppMsg::ScrollUp, // 特殊处理
        AppState::SelectingMoveTarget { .. } => AppMsg::MoveTargetNavUp, // 可能需要新增
        AppState::Cloning { .. } => AppMsg::ClonePreviousMainDir,
        _ => AppMsg::PreviousRepo, // 默认
    }
}

// 根据当前状态返回对应的向下导航消息
fn get_scroll_down_message(app: &App) -> AppMsg {
    match &app.state {
        AppState::Running => AppMsg::NextRepo,
        AppState::ChoosingDir { .. } => AppMsg::DirectoryNavDown,
        AppState::SelectingTheme { .. } => AppMsg::ThemeNavDown,
        AppState::ManagingDirs { .. } => AppMsg::MainDirNavDown,
        AppState::ShowingHelp { .. } => AppMsg::ScrollDown, // 特殊处理
        AppState::SelectingMoveTarget { .. } => AppMsg::MoveTargetNavDown, // 可能需要新增
        AppState::Cloning { .. } => AppMsg::CloneNextMainDir,
        _ => AppMsg::NextRepo, // 默认
    }
}
```

### Task 2: 检查并补充消息类型

**文件**: `src/app/msg.rs`

**检查项**:
- [ ] `MoveTargetNavUp` / `MoveTargetNavDown` 是否存在
- [ ] 确保所有导航消息都已定义

### Task 3: 确保 `update.rs` 处理所有导航消息

**文件**: `src/app/update.rs`

**检查项**:
- [ ] `AppMsg::PreviousRepo` / `NextRepo` - 已存在
- [ ] `AppMsg::DirectoryNavUp` / `DirectoryNavDown` - 已存在
- [ ] `AppMsg::ThemeNavUp` / `ThemeNavDown` - 已存在
- [ ] `AppMsg::MainDirNavUp` / `MainDirNavDown` - 已存在
- [ ] `AppMsg::ScrollUp` / `ScrollDown` - 已存在（帮助面板需要特殊处理）
- [ ] `AppMsg::ClonePreviousMainDir` / `CloneNextMainDir` - 已存在
- [ ] `AppMsg::MoveTargetNavUp` / `MoveTargetNavDown` - 如新增则需实现

### Task 4: 帮助面板特殊处理

**文件**: `src/app/update.rs`

帮助面板的滚动是直接修改 `scroll_offset`，需要确保 `ScrollUp`/`ScrollDown` 消息正确处理：

```rust
AppMsg::ScrollUp => {
    if let AppState::ShowingHelp { scroll_offset } = &mut app.state {
        if *scroll_offset > 0 {
            *scroll_offset = scroll_offset.saturating_sub(1);
        }
    }
}
AppMsg::ScrollDown => {
    if let AppState::ShowingHelp { scroll_offset } = &mut app.state {
        *scroll_offset = scroll_offset.saturating_add(1);
        // 需要限制最大值
    }
}
```

### Task 5: 移动目标选择器支持

**文件**: `src/app/msg.rs` + `src/app/update.rs`

如 `MoveTargetNavUp`/`MoveTargetNavDown` 不存在，可选择：
- **方案 A**: 新增消息类型
- **方案 B**: 直接在 mouse handler 中修改 `ListState`

推荐**方案 B**，简化实现：

```rust
// 在 handle_mouse_event 中直接处理
AppState::SelectingMoveTarget { list_state, .. } => {
    let current = list_state.selected().unwrap_or(0);
    if event.kind == MouseEventKind::ScrollUp && current > 0 {
        list_state.select(Some(current - 1));
    } else if event.kind == MouseEventKind::ScrollDown && current < max {
        list_state.select(Some(current + 1));
    }
}
```

---

## 测试验证

### 手动测试清单
- [ ] 主仓库列表滚轮滚动（逐行）
- [ ] 目录选择器滚轮滚动（逐行）
- [ ] 主题选择器滚轮滚动（逐行）
- [ ] 主目录管理器滚轮滚动（逐行）
- [ ] 帮助面板滚轮滚动（逐行）
- [ ] 移动目标选择滚轮滚动（逐行）
- [ ] 克隆对话框目录选择滚轮滚动（逐行）

### 编译检查
```bash
cargo build
cargo clippy -- -D warnings
```

---

## 风险与注意事项

1. **终端兼容性**: 不同终端的滚轮事件可能略有差异，需测试主流终端
2. **边界处理**: 滚动到顶部/底部时的行为需一致（循环或停止）
3. **性能影响**: 滚轮事件频繁触发，确保处理逻辑轻量

---

## 执行命令

```bash
# 开发构建
cargo build

# 运行测试
cargo test

# Lint 检查
cargo clippy -- -D warnings

# 运行应用
cargo run
```

---

## 相关文件

- `src/handler/mouse.rs` - 鼠标事件处理（主要修改）
- `src/handler/keyboard.rs` - 键盘事件处理（参考）
- `src/app/msg.rs` - 消息类型定义
- `src/app/update.rs` - 状态更新逻辑
- `src/app/state.rs` - 应用状态定义
