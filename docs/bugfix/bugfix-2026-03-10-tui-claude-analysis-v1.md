# Bug 分析报告

## 基本信息

- **Bug ID**: TUI-CLAUDE-001
- **日期**: 2026-03-10
- **报告人**: Bug Analyst

## 问题描述

当在 repotui 中触发 "Open in Claude Code" action 时，Claude Code 无法正常显示。TUI 程序保持运行但 Claude Code 的界面被干扰，导致用户无法正常使用。

## 根因分析

### 终端状态分析

TUI 程序启动时会修改终端状态：

1. **Raw Mode** (`src/lib.rs:59`)
   - 启用原始模式，键盘输入不经过系统处理
   - 直接读取按键事件

2. **Alternate Screen** (`src/lib.rs:61`)
   - 切换到备用屏幕（alternate screen buffer）
   - 退出时恢复到主屏幕

3. **隐藏光标** (`src/lib.rs:64`)
   - 隐藏终端光标

### Action 执行分析

当前执行逻辑 (`src/action/execute.rs:83-99`):

```rust
fn execute_cd_and_cloud(repo_path: &Path) -> AppResult<()> {
    let claude_path = which::which("claude")
        .map_err(|_| ActionError::CommandNotFound("claude".to_string()))?;

    // 问题：直接在 TUI 的终端状态下启动 Claude
    let status = Command::new(claude_path)
        .current_dir(repo_path)
        .status()?;  // 继承父进程的 stdin/stdout/stderr
    ...
}
```

**问题根源**:
- Claude Code 子进程继承了 TUI 的终端状态（raw mode + alternate screen）
- Claude 期望正常的终端交互环境
- 两个程序同时竞争终端显示，导致界面异常

## 影响范围评估

### 受影响的功能
- "Open in Claude Code" (Action::CdAndCloud)
- "Open in OpenCode" (Action::OpenOpenCode)

### 受影响的用户场景
- 用户从 TUI 启动 Claude Code 进行代码编辑
- 用户无法正常使用交互式命令行工具

### 严重程度
- **严重 (High)** - 核心功能无法正常使用

## 修复目标

在启动外部交互式命令（如 Claude Code）前：
1. 恢复终端到正常状态（disable raw mode, leave alternate screen）
2. 等待命令执行完成
3. 重新初始化 TUI 终端状态

## 相关代码文件

- `src/action/execute.rs` - Action 执行逻辑
- `src/lib.rs` - 终端初始化和恢复
