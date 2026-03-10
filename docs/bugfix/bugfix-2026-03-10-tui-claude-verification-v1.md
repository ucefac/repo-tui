# Bug 修复验证报告

## Bug 信息

- **Bug ID**: TUI-CLAUDE-001
- **标题**: "Open in Claude Code" action 执行后 Claude 无法正常显示
- **修复日期**: 2026-03-10

## 修复内容

### 问题根因

当 TUI 程序启动时，会修改终端状态：
- 启用 raw mode
- 切换到 alternate screen
- 隐藏光标

执行 "Open in Claude Code" action 时，Claude Code 子进程继承了这些终端状态，导致无法正常显示和交互。

### 修复方案

**核心思路**: 在执行交互式 CLI 命令前恢复终端状态，执行完成后重新初始化 TUI。

### 修改的文件

1. **src/error.rs**
   - 添加 `ActionError::TerminalNeedsReinit` 错误类型
   - 用于标记终端需要重新初始化

2. **src/action/execute.rs**
   - 修改 `execute_cd_and_cloud()` 函数
   - 修改 `execute_cd_and_opencode()` 函数
   - 执行前: 调用 `disable_raw_mode()` 和 `LeaveAlternateScreen`
   - 执行后: 返回 `TerminalNeedsReinit` 错误

3. **src/app/msg.rs**
   - 添加 `AppMsg::TerminalNeedsReinit` 消息类型

4. **src/app/update.rs**
   - 修改 `ActionExecuted` 处理逻辑
   - 检测到 `TerminalNeedsReinit` 时发送重新初始化消息

5. **src/lib.rs**
   - 主循环处理 `TerminalNeedsReinit` 消息
   - 调用 `init_terminal()` 重新初始化终端

## 验证结果

### 编译检查

```bash
$ cargo check
    Finished `dev` profile [unoptimized + debug info] target(s) in 2.01s
```

✅ 编译通过

### 单元测试

```bash
$ cargo test
...
test result: ok. 284 passed; 0 failed
```

✅ 所有 284 个测试通过

### Lint 检查

```bash
$ cargo clippy -- -D warnings
```

✅ 无警告

## 行为变化

### 修复前
- 在 TUI 中选择 "Open in Claude Code"
- Claude Code 启动但显示异常
- TUI 仍在后台运行，终端状态混乱

### 修复后
- 在 TUI 中选择 "Open in Claude Code"
- TUI 界面消失，终端恢复正常状态
- Claude Code 正常显示，可以交互
- 退出 Claude Code 后，TUI 自动恢复

## 回归测试

- [x] 其他 action (Open in VS Code, Open in WebStorm 等) 不受影响
- [x] 正常 TUI 操作不受影响
- [x] 终端状态正确恢复

## 结论

✅ 修复成功，Bug 已解决。
