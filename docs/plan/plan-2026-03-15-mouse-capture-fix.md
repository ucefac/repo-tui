# 鼠标捕获修复计划

**日期**: 2026-03-15  
**任务**: 修复鼠标滚轮未被 TUI 正确捕获的问题  
**优先级**: 高  
**前置任务**: mouse-wheel-scroll (已完成)

---

## 问题描述

用户反馈鼠标滚轮在 TUI 中无法正常工作：

1. **Terminal.app**: 滚轮滚动整个终端窗口，而非 TUI 内部的列表
2. **kakuku**: 滚轮一次滚动 3-4 行，而非逐行滚动（每次 1 行）

## 根因分析

`src/lib.rs` 的 `init_terminal()` 函数中**未启用 `EnableMouseCapture`**：

```rust
// 当前代码（有问题）
execute!(stdout, EnterAlternateScreen, EnableBracketedPaste)?;
```

这导致：
1. 鼠标事件未被 TUI 应用捕获，终端自己处理滚轮事件
2. Terminal.app 滚动整个终端窗口
3. kakuku 使用终端默认的滚动行为（一次 3-4 行）

## 解决方案

在终端初始化时启用鼠标捕获，确保所有鼠标事件（包括滚轮）都被传递给 TUI 应用处理。

---

## 任务分解

### Task 1: 修改 `src/lib.rs` - 启用鼠标捕获

**目标**: 在 `init_terminal()` 和 `restore_terminal()` 中添加鼠标捕获支持

**修改内容**:

1. **更新 import** (第 38-42 行):
```rust
use crossterm::{
    event::{self, EnableBracketedPaste, EnableMouseCapture, DisableMouseCapture, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
```

2. **修改 `init_terminal()`** (第 58-67 行):
```rust
fn init_terminal() -> Result<Terminal<CrosstermBackend<io::Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    // 添加 EnableMouseCapture
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture, EnableBracketedPaste)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.hide_cursor()?;
    Ok(terminal)
}
```

3. **修改 `restore_terminal()`** (第 69-74 行):
```rust
fn restore_terminal() -> Result<()> {
    disable_raw_mode()?;
    // 添加 DisableMouseCapture
    execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture)?;
    Ok(())
}
```

**验收标准**:
- [ ] 编译通过 (`cargo build`)
- [ ] Clippy 检查通过 (`cargo clippy -- -D warnings`)
- [ ] 无新的测试失败
- [ ] Terminal.app 中滚轮滚动 TUI 列表，而非终端窗口
- [ ] kakuku 中滚轮逐行滚动（每次 1 行）

---

## 测试验证

### 手动测试清单

在以下终端中测试滚轮滚动：

- [ ] **Terminal.app** (macOS):
  - 滚动主仓库列表 - 应逐行滚动，不滚动终端窗口
  - 滚动目录选择器 - 应逐行滚动
  - 滚动主题选择器 - 应逐行滚动
  - 滚动主目录管理器 - 应逐行滚动
  - 滚动帮助面板 - 应逐行滚动

- [ ] **kakuku**:
  - 滚动主仓库列表 - 应逐行滚动（每次 1 行，而非 3-4 行）
  - 滚动其他列表 - 应逐行滚动

- [ ] **iTerm2** (如可用):
  - 滚动主仓库列表 - 应逐行滚动

### 编译检查

```bash
cargo build
cargo clippy -- -D warnings
cargo test
```

---

## 风险与注意事项

1. **终端兼容性**: 不同终端的鼠标事件实现可能略有差异，需测试主流终端
2. **现有功能影响**: 确保鼠标点击事件（如路径栏点击）仍然正常工作
3. **回滚方案**: 如发现问题，移除 `EnableMouseCapture` 即可恢复原状

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

- `src/lib.rs` - 终端初始化（主要修改）
- `src/handler/mouse.rs` - 鼠标事件处理（已由 mouse-wheel-scroll 任务实现）
- `src/app/update.rs` - 导航消息处理（已由 mouse-wheel-scroll 任务实现）

---

## 依赖关系

本计划依赖于已完成的 `mouse-wheel-scroll` 计划：
- ✅ `mouse-wheel-scroll` 已实现滚轮事件处理和导航逻辑
- 🔧 本计划启用鼠标捕获，使滚轮事件能被 TUI 正确接收

---

## 执行报告位置

执行完成后，报告应保存到：`./docs/reports/mouse-capture-fix-report.md`
