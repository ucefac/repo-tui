# 修复方案设计

## 方案对比

| 方案 | 描述 | 优点 | 缺点 | 推荐度 |
|------|------|------|------|--------|
| **方案 A** | 执行前恢复终端，执行后重新初始化 | 简单直接，TUI 完全释放终端 | TUI 界面暂时消失 | ⭐⭐⭐⭐⭐ |
| **方案 B** | 使用子进程单独分配终端 | 保持 TUI 显示 | 复杂，需要新终端窗口 | ⭐⭐ |

## 推荐方案：方案 A

### 核心思路

在 `execute_cd_and_cloud` 函数中：

1. **执行前**: 调用 `restore_terminal()` 恢复终端
2. **执行**: 启动 Claude Code（此时终端是正常状态）
3. **执行后**: 调用 `init_terminal()` 重新初始化终端

### 具体实现

修改 `src/action/execute.rs`：

```rust
use crossterm::{
    execute,
    terminal::{disable_raw_mode, LeaveAlternateScreen},
};
use std::io::{self, Write};

fn execute_cd_and_cloud(repo_path: &Path) -> AppResult<()> {
    // 1. 恢复终端到正常状态
    let _ = disable_raw_mode();
    let _ = execute!(io::stdout(), LeaveAlternateScreen);
    let _ = io::stdout().flush();

    // 2. 执行 Claude Code
    let claude_path = which::which("claude")
        .map_err(|_| ActionError::CommandNotFound("claude".to_string()))?;

    let status = Command::new(claude_path)
        .current_dir(repo_path)
        .status()?;

    // 3. 等待用户按键后恢复 TUI
    println!("\nPress any key to return to repotui...");
    let _ = crossterm::event::read();

    // 4. 重新初始化终端（由主循环处理）
    // 由于无法直接从 action 模块访问 terminal 实例，
    // 需要通过错误码或特殊返回值通知主循环重新初始化

    if !status.success() {
        return Err(AppError::Action(ActionError::ExecutionFailed(...)));
    }

    Ok(())
}
```

### 关键问题

**问题**: `action` 模块无法直接访问 `Terminal` 实例进行重新初始化。

**解决方案**:
1. 添加一个新的错误类型 `AppError::TerminalNeedsReinit`
2. 主循环捕获此错误后重新初始化终端
3. 或者修改 `ActionError` 添加标记字段

### 修改的文件

1. `src/action/execute.rs` - 修改执行逻辑
2. `src/error.rs` - 添加需要重新初始化终端的错误类型
3. `src/lib.rs` - 主循环处理终端重新初始化

## 风险评估

| 风险 | 可能性 | 影响 | 缓解措施 |
|------|--------|------|----------|
| 终端恢复失败 | 低 | 中 | 使用 `let _ =` 忽略错误，继续执行 |
| 用户未按键导致无法返回 | 低 | 低 | 明确提示用户 |
| 重新初始化失败 | 极低 | 高 | 优雅退出程序 |

## 测试策略

1. **手动测试**: 在 TUI 中触发 action，验证 Claude 正常显示
2. **回归测试**: 验证其他 action 不受影响
3. **边界测试**: 验证 Claude 退出代码非 0 时的行为

## 实施步骤

1. 修改 `src/error.rs` 添加新错误类型
2. 修改 `src/action/execute.rs` 实现终端切换逻辑
3. 修改 `src/lib.rs` 处理终端重新初始化
4. 运行测试验证修复
